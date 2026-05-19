use actix_web::{web, App, HttpResponse, HttpServer};
use lapin::{
    options::{BasicPublishOptions, ExchangeDeclareOptions},
    types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties, ExchangeKind,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    env,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
struct AppState {
    amqp_url: String,
    ifas: Arc<Mutex<HashMap<String, IfaConfig>>>,
}

#[derive(Clone, Deserialize, Serialize)]
struct IfaConfig {
    ifa_id: String,
    events: HashMap<String, EventChannel>,
}

#[derive(Clone, Deserialize, Serialize)]
struct EventChannel {
    exchange: String,
    routing_key: String,
}

#[derive(Deserialize, Serialize)]
struct ClientMessage {
    message_id: String,
    correlation_id: String,
    ifa_id: String,
    name: String,
    #[serde(default)]
    context: Value,
    #[serde(default)]
    payload: Value,
}

#[derive(Serialize)]
struct AcceptedMessage {
    status: &'static str,
    lifecycle_state: &'static str,
    message_id: String,
    correlation_id: String,
    exchange: String,
    routing_key: String,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let amqp_url = env::var("MESSAGEBRIDGE_AMQP_URL")
        .unwrap_or_else(|_| "amqp://guest:guest@localhost:5672/%2f".to_string());
    let http_bind =
        env::var("MESSAGEBRIDGE_HTTP_BIND").unwrap_or_else(|_| "0.0.0.0:8082".to_string());

    connect_channel(&amqp_url)
        .await
        .expect("RabbitMQ did not become ready");
    let state = AppState {
        amqp_url,
        ifas: Arc::new(Mutex::new(HashMap::new())),
    };

    log::info!("Starting messagebridge HTTP API on {}", http_bind);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .route("/health", web::get().to(health))
            .route("/admin/config", web::delete().to(reset_config))
            .route("/admin/config/ifas", web::post().to(register_ifa))
            .route("/messages", web::post().to(publish_message))
    })
    .bind(http_bind)?
    .run()
    .await
}

async fn connect_channel(amqp_url: &str) -> Result<Channel, String> {
    let mut last_error = None;

    for _ in 0..60 {
        match Connection::connect(amqp_url, ConnectionProperties::default()).await {
            Ok(connection) => match connection.create_channel().await {
                Ok(channel) => return Ok(channel),
                Err(err) => last_error = Some(err.to_string()),
            },
            Err(err) => last_error = Some(err.to_string()),
        }

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    Err(format!("RabbitMQ did not become ready: {last_error:?}"))
}

async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({ "status": "OK" }))
}

async fn reset_config(state: web::Data<AppState>) -> HttpResponse {
    let mut ifas = state.ifas.lock().expect("ifa config mutex poisoned");
    ifas.clear();
    HttpResponse::Ok().json(serde_json::json!({ "status": "Config reset successfully" }))
}

async fn register_ifa(state: web::Data<AppState>, config: web::Json<IfaConfig>) -> HttpResponse {
    let channel = match connect_channel(&state.amqp_url).await {
        Ok(channel) => channel,
        Err(err) => {
            log::error!(
                "Could not connect to RabbitMQ while registering IFA: {}",
                err
            );
            return HttpResponse::InternalServerError()
                .content_type("text/plain; charset=utf-8")
                .body(format!("Could not connect to RabbitMQ: {}", err));
        }
    };

    for event_channel in config.events.values() {
        if let Err(err) = ensure_exchange(&channel, &event_channel.exchange).await {
            log::error!(
                "Could not declare exchange {}: {}",
                event_channel.exchange,
                err
            );
            return HttpResponse::InternalServerError()
                .content_type("text/plain; charset=utf-8")
                .body(format!("Could not declare exchange: {}", err));
        }
    }

    let mut ifas = state.ifas.lock().expect("ifa config mutex poisoned");
    ifas.insert(config.ifa_id.clone(), config.into_inner());
    HttpResponse::Created().json(serde_json::json!({ "status": "registered" }))
}

async fn publish_message(
    state: web::Data<AppState>,
    message: web::Json<ClientMessage>,
) -> HttpResponse {
    let channel_config = {
        let ifas = state.ifas.lock().expect("ifa config mutex poisoned");
        let Some(ifa) = ifas.get(&message.ifa_id) else {
            return HttpResponse::NotFound()
                .content_type("text/plain; charset=utf-8")
                .body(format!("Unknown IFA config: {}", message.ifa_id));
        };
        let Some(channel_config) = ifa.events.get(&message.name) else {
            return HttpResponse::NotFound()
                .content_type("text/plain; charset=utf-8")
                .body(format!("Unknown event channel: {}", message.name));
        };
        channel_config.clone()
    };

    let payload = match serde_json::to_vec(&*message) {
        Ok(payload) => payload,
        Err(err) => {
            return HttpResponse::BadRequest()
                .content_type("text/plain; charset=utf-8")
                .body(format!("Invalid message: {}", err));
        }
    };

    let channel = match connect_channel(&state.amqp_url).await {
        Ok(channel) => channel,
        Err(err) => {
            log::error!(
                "Could not connect to RabbitMQ while publishing message: {}",
                err
            );
            return HttpResponse::InternalServerError()
                .content_type("text/plain; charset=utf-8")
                .body(format!("Could not connect to RabbitMQ: {}", err));
        }
    };

    if let Err(err) = ensure_exchange(&channel, &channel_config.exchange).await {
        log::error!(
            "Could not declare exchange {}: {}",
            channel_config.exchange,
            err
        );
        return HttpResponse::InternalServerError()
            .content_type("text/plain; charset=utf-8")
            .body(format!("Could not declare exchange: {}", err));
    }

    let confirm = channel
        .basic_publish(
            &channel_config.exchange,
            &channel_config.routing_key,
            BasicPublishOptions::default(),
            &payload,
            BasicProperties::default()
                .with_content_type("application/json".into())
                .with_message_id(message.message_id.clone().into())
                .with_correlation_id(message.correlation_id.clone().into())
                .with_delivery_mode(2),
        )
        .await;

    let confirm = match confirm {
        Ok(confirm) => confirm.await,
        Err(err) => {
            log::error!("Could not publish message: {}", err);
            return HttpResponse::InternalServerError()
                .content_type("text/plain; charset=utf-8")
                .body(format!("Could not publish message: {}", err));
        }
    };

    match confirm {
        Ok(_) => HttpResponse::Accepted().json(AcceptedMessage {
            status: "accepted",
            lifecycle_state: "delivered",
            message_id: message.message_id.clone(),
            correlation_id: message.correlation_id.clone(),
            exchange: channel_config.exchange,
            routing_key: channel_config.routing_key,
        }),
        Err(err) => {
            log::error!("Publish was not confirmed: {}", err);
            HttpResponse::InternalServerError()
                .content_type("text/plain; charset=utf-8")
                .body(format!("Publish was not confirmed: {}", err))
        }
    }
}

async fn ensure_exchange(channel: &Channel, exchange: &str) -> lapin::Result<()> {
    channel
        .exchange_declare(
            exchange,
            ExchangeKind::Topic,
            ExchangeDeclareOptions {
                durable: true,
                ..ExchangeDeclareOptions::default()
            },
            FieldTable::default(),
        )
        .await
}
