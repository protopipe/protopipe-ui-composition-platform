use actix_web::{web, App, HttpResponse, HttpServer};
use futures_util::StreamExt;
use lapin::{
    options::{
        BasicAckOptions, BasicConsumeOptions, ExchangeDeclareOptions, QueueBindOptions,
        QueueDeclareOptions,
    },
    types::FieldTable,
    Connection, ConnectionProperties, ExchangeKind,
};
use serde::Serialize;
use serde_json::Value;
use std::{
    collections::HashMap,
    env,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    processed: Arc<Mutex<Vec<ProcessedMessage>>>,
}

#[derive(Clone, Serialize)]
struct ProcessedMessage {
    id: Uuid,
    queue: String,
    delivery_tag: u64,
    message_id: Option<String>,
    correlation_id: Option<String>,
    message_name: Option<String>,
    routing_key: String,
    payload: Value,
    headers: HashMap<String, String>,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let amqp_url = env::var("MESSAGE_WORKER_MOCK_AMQP_URL")
        .unwrap_or_else(|_| "amqp://guest:guest@localhost:5672/%2f".to_string());
    let queues = env::var("MESSAGE_WORKER_MOCK_QUEUES")
        .unwrap_or_else(|_| "protopipe.commands".to_string())
        .split(',')
        .map(str::trim)
        .filter(|queue| !queue.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let bindings = env::var("MESSAGE_WORKER_MOCK_BINDINGS")
        .unwrap_or_else(|_| "protopipe.commands:protopipe.commands:#".to_string())
        .split(',')
        .map(str::trim)
        .filter(|binding| !binding.is_empty())
        .filter_map(parse_binding)
        .collect::<Vec<_>>();
    let http_bind =
        env::var("MESSAGE_WORKER_MOCK_HTTP_BIND").unwrap_or_else(|_| "0.0.0.0:9100".to_string());

    let state = AppState {
        processed: Arc::new(Mutex::new(Vec::new())),
    };

    for queue in queues {
        let consumer_state = state.clone();
        let consumer_amqp_url = amqp_url.clone();
        let consumer_bindings = bindings.clone();
        tokio::spawn(async move {
            loop {
                if let Err(err) = consume_queue(
                    consumer_amqp_url.clone(),
                    queue.clone(),
                    consumer_bindings.clone(),
                    consumer_state.clone(),
                )
                .await
                {
                    log::warn!(
                        "Message worker mock consumer for queue {} stopped: {}. Retrying.",
                        queue,
                        err
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }
        });
    }

    log::info!("Starting message-worker-mock HTTP API on {}", http_bind);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .route("/health", web::get().to(health))
            .route("/processed", web::get().to(get_processed))
            .route("/processed", web::delete().to(clear_processed))
    })
    .bind(http_bind)?
    .run()
    .await
}

async fn consume_queue(
    amqp_url: String,
    queue: String,
    bindings: Vec<QueueBinding>,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let connection = Connection::connect(&amqp_url, ConnectionProperties::default()).await?;
    let channel = connection.create_channel().await?;

    channel
        .queue_declare(
            &queue,
            QueueDeclareOptions {
                durable: true,
                ..QueueDeclareOptions::default()
            },
            FieldTable::default(),
        )
        .await?;

    for binding in bindings.iter().filter(|binding| binding.queue == queue) {
        channel
            .exchange_declare(
                &binding.exchange,
                ExchangeKind::Topic,
                ExchangeDeclareOptions {
                    durable: true,
                    ..ExchangeDeclareOptions::default()
                },
                FieldTable::default(),
            )
            .await?;
        channel
            .queue_bind(
                &binding.queue,
                &binding.exchange,
                &binding.routing_key,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await?;
    }

    let mut consumer = channel
        .basic_consume(
            &queue,
            "message-worker-mock",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    log::info!("Consuming RabbitMQ queue {}", queue);

    while let Some(delivery_result) = consumer.next().await {
        let delivery = delivery_result?;
        let processed = processed_message(&queue, &delivery);

        {
            let mut messages = state
                .processed
                .lock()
                .expect("processed log mutex poisoned");
            messages.push(processed);
        }

        delivery.ack(BasicAckOptions::default()).await?;
    }

    Ok(())
}

#[derive(Clone)]
struct QueueBinding {
    queue: String,
    exchange: String,
    routing_key: String,
}

fn parse_binding(value: &str) -> Option<QueueBinding> {
    let mut parts = value.splitn(3, ':');
    Some(QueueBinding {
        queue: parts.next()?.to_string(),
        exchange: parts.next()?.to_string(),
        routing_key: parts.next()?.to_string(),
    })
}

fn processed_message(queue: &str, delivery: &lapin::message::Delivery) -> ProcessedMessage {
    let payload = serde_json::from_slice::<Value>(&delivery.data)
        .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&delivery.data).into_owned()));
    let message_id = delivery
        .properties
        .message_id()
        .as_ref()
        .map(ToString::to_string)
        .or_else(|| json_string_field(&payload, "message_id"));
    let correlation_id = delivery
        .properties
        .correlation_id()
        .as_ref()
        .map(ToString::to_string)
        .or_else(|| json_string_field(&payload, "correlation_id"));
    let message_name = json_string_field(&payload, "name");

    ProcessedMessage {
        id: Uuid::new_v4(),
        queue: queue.to_string(),
        delivery_tag: delivery.delivery_tag,
        message_id,
        correlation_id,
        message_name,
        routing_key: delivery.routing_key.to_string(),
        payload,
        headers: HashMap::new(),
    }
}

fn json_string_field(payload: &Value, field: &str) -> Option<String> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({ "status": "OK" }))
}

async fn get_processed(state: web::Data<AppState>) -> HttpResponse {
    let messages = state
        .processed
        .lock()
        .expect("processed log mutex poisoned");
    HttpResponse::Ok().json(messages.clone())
}

async fn clear_processed(state: web::Data<AppState>) -> HttpResponse {
    let mut messages = state
        .processed
        .lock()
        .expect("processed log mutex poisoned");
    messages.clear();
    HttpResponse::Ok().json(serde_json::json!({ "status": "cleared" }))
}
