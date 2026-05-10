use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::AppState;

/// A static data value, deserialized as-is from configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticData {
    pub value: serde_json::Value,
}

/// A dynamic REST-backed data value.
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

pub type PageDataDto = HashMap<String, DataValue>;

#[derive(Serialize, Deserialize)]
pub struct PageConfigDto {
    pub path: String,
    pub page_id: String,
    pub template: String,
    pub rfa: String,
    pub timeout_ms: u64,
    pub data: PageDataDto,
}

#[derive(Serialize, Deserialize)]
pub struct RFAConfigDto {
    pub id: String,
    pub source: String,
    pub version: String,
}

pub fn resolve_page(state: &AppState, path: &str) -> Option<PageConfig> {
    let pages = state.pages.lock().unwrap();
    pages.get(path).cloned()
}

pub fn resolve_rfa(state: &AppState, rfa_id: &str) -> Option<RFAConfig> {
    let rfas = state.rfas.lock().unwrap();
    rfas.get(rfa_id).cloned()
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
        data: config.data.clone().into_iter().collect(),
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

    if let Err(err) = state.render_pool.register_rfa(&rfa_config).await {
        log::error!("Failed to register RFA in render pool: {}", err);
        return HttpResponse::InternalServerError()
            .content_type("text/plain; charset=utf-8")
            .body(format!("Failed to register RFA: {}", err));
    }

    let mut rfas = state.rfas.lock().unwrap();
    rfas.insert(config.id.clone(), rfa_config);

    log::info!("Registered RFA: {}", config.id);
    HttpResponse::Created().json(config.into_inner())
}
