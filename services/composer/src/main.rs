use actix_web::{web, App};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

mod admin;
mod render;

/// A static data value, deserialized as-is from configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticData {
    pub value: serde_json::Value,
}

/// A dynamic REST-backed data value. Implemented later.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DynamicRestData {
    pub endpoint: String,
    pub default: serde_json::Value,
}

/// A typed data value for page config.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum DataValue {
    Static(StaticData),
    DynamicRest(DynamicRestData),
}

/// Page configuration with structured data values.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PageConfig {
    pub path: String,
    pub page_id: String,
    pub template: String,
    pub rfa: String,
    pub timeout_ms: u64,
    pub data: HashMap<String, DataValue>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RFAConfig {
    pub id: String,
    pub source: String,
    pub version: String,
}

pub struct AppState {
    pub pages: Mutex<HashMap<String, PageConfig>>,
    pub rfas: Mutex<HashMap<String, RFAConfig>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let state = web::Data::new(AppState {
        pages: Mutex::new(HashMap::new()),
        rfas: Mutex::new(HashMap::new()),
    });

    log::info!("Starting Composer Service");
    log::info!("Admin server on http://0.0.0.0:9000");
    log::info!("Render server on http://0.0.0.0:8080");

    // Admin Server (Port 9000)
    let admin_state = state.clone();
    let admin_server = actix_web::HttpServer::new(move || {
        App::new()
            .app_data(admin_state.clone())
            .service(
                web::scope("/admin")
                    .route("/health", web::get().to(admin::health))
                    .route("/config/pages", web::post().to(admin::register_page))
                    .route("/rfa/register", web::post().to(admin::register_rfa))
            )
    })
    .bind("0.0.0.0:9000")?
    .run();

    // Render Server (Port 8080)
    let render_state = state.clone();
    let render_server = actix_web::HttpServer::new(move || {
        App::new()
            .app_data(render_state.clone())
            .default_service(web::route().to(render::render_page))
    })
    .bind("0.0.0.0:8080")?
    .run();

    // Run both servers concurrently
    futures::future::try_join(admin_server, render_server).await?;

    Ok(())
}
