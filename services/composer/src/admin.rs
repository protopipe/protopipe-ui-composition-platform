use crate::{AppState, PageConfig, RFAConfig};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PageConfigDto {
    pub path: String,
    pub page_id: String,
    pub template: String,
    pub rfa: String,
    pub timeout_ms: u64,
    pub defaults: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct RFAConfigDto {
    pub id: String,
    pub source: String,
    pub version: String,
}

pub async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"status": "OK"}))
}

pub async fn register_page(
    state: web::Data<AppState>,
    config: web::Json<PageConfigDto>,
) -> HttpResponse {
    let page_config = PageConfig {
        path: config.path.clone(),
        page_id: config.page_id.clone(),
        template: config.template.clone(),
        rfa: config.rfa.clone(),
        timeout_ms: config.timeout_ms,
        defaults: config.defaults.clone(),
    };

    let mut pages = state.pages.lock().unwrap();
    pages.insert(config.path.clone(), page_config);

    log::info!("Registered page: {}", config.path);
    HttpResponse::Created().json(config.into_inner())
}

pub async fn register_rfa(
    state: web::Data<AppState>,
    config: web::Json<RFAConfigDto>,
) -> HttpResponse {
    let rfa_config = RFAConfig {
        id: config.id.clone(),
        source: config.source.clone(),
        version: config.version.clone(),
    };

    let mut rfas = state.rfas.lock().unwrap();
    rfas.insert(config.id.clone(), rfa_config);

    log::info!("Registered RFA: {}", config.id);
    HttpResponse::Created().json(config.into_inner())
}
