use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::page_resolver::route_key;
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

/// A runtime value sourced from one GET parameter.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetParameterData {
    pub key: String,
}

/// A REST-backed data value resolved through a registered Composer service.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RestServiceData {
    pub service: String,
    pub path: String,
    pub method: Option<String>,
    pub timeout_ms: Option<u64>,
    pub error_default: Option<serde_json::Value>,
}

/// A single side-effecting service call used to process a submitted request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmitServiceData {
    pub target: String,
    pub service: String,
    pub path: String,
    pub method: String,
    pub content_type: String,
    pub timeout_ms: Option<u64>,
    pub error_default: Option<serde_json::Value>,
}

/// A typed data value for page config.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum DataValue {
    Static(StaticData),
    DynamicRest(DynamicRestData),
    #[serde(alias = "restService")]
    RestService(RestServiceData),
    Url,
    #[serde(alias = "getParameter")]
    GetParameter(GetParameterData),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PageType {
    Rfa,
    Ifa,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum PageDelivery {
    Composer,
    UpstreamProxy {
        origin: String,
        #[serde(default)]
        markers: Vec<ProxyMarkerReplacement>,
    },
}

impl Default for PageDelivery {
    fn default() -> Self {
        Self::Composer
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProxyMarkerReplacement {
    pub id: String,
    pub rfa: String,
    #[serde(default = "default_proxy_marker_fallback")]
    pub fallback: String,
}

fn default_proxy_marker_fallback() -> String {
    "keep-upstream".to_string()
}

/// Page configuration with structured data values.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PageConfig {
    pub path: String,
    pub method: String,
    pub page_id: String,
    #[serde(rename = "type")]
    pub page_type: PageType,
    pub template: String,
    pub rfa: String,
    pub delivery: PageDelivery,
    pub timeout_ms: u64,
    pub content_type: String,
    pub submit: Option<SubmitServiceData>,
    pub data: HashMap<String, DataValue>,
    pub interaction: Option<serde_json::Value>,
}

pub type PageDataDto = HashMap<String, DataValue>;

#[derive(Serialize, Deserialize)]
pub struct PageConfigDto {
    pub path: String,
    pub method: Option<String>,
    pub page_id: String,
    #[serde(rename = "type")]
    pub page_type: PageType,
    pub template: Option<String>,
    pub rfa: Option<String>,
    pub delivery: Option<PageDelivery>,
    pub timeout_ms: u64,
    pub content_type: Option<String>,
    pub submit: Option<SubmitServiceData>,
    pub data: Option<PageDataDto>,
    pub interaction: Option<serde_json::Value>,
}

pub async fn register_page(
    state: web::Data<AppState>,
    config: web::Json<PageConfigDto>,
) -> HttpResponse {
    if let Err(message) = validate_page_config_dto(&config) {
        return HttpResponse::BadRequest()
            .content_type("text/plain; charset=utf-8")
            .body(message);
    }

    let page_config = page_config_from_dto(&config);

    let mut pages = state.pages.lock().unwrap();
    pages.insert(
        route_key(&page_config.method, &page_config.path),
        page_config,
    );

    log::info!("Registered page: {}", config.path);
    HttpResponse::Created().json(config.into_inner())
}

pub async fn get_pages(state: web::Data<AppState>) -> HttpResponse {
    let pages = state.pages.lock().unwrap();
    let page_list: Vec<PageConfigDto> = pages.values().map(page_config_to_dto).collect();

    HttpResponse::Ok().json(page_list)
}

pub async fn reset_pages(state: web::Data<AppState>) {
    let mut pages = state.pages.lock().unwrap();
    pages.clear();
}

pub fn validate_page_config(config: &PageConfig) -> Result<(), &'static str> {
    validate_page_delivery(config)?;
    validate_submit_config(&config.method, config.submit.as_ref())
}

fn validate_page_config_dto(config: &PageConfigDto) -> Result<(), &'static str> {
    validate_page_delivery_dto(config)?;
    validate_page_interaction(&config.page_type, config.interaction.is_some())?;
    validate_submit_config(
        &page_method(config.method.as_deref()),
        config.submit.as_ref(),
    )
}

fn validate_page_delivery(config: &PageConfig) -> Result<(), &'static str> {
    match &config.delivery {
        PageDelivery::Composer if config.rfa.is_empty() => Err("Composer pages must define an RFA"),
        PageDelivery::UpstreamProxy { origin, .. } if origin.is_empty() => {
            Err("Proxy pages must define an upstream origin")
        }
        _ => Ok(()),
    }
}

fn validate_page_delivery_dto(config: &PageConfigDto) -> Result<(), &'static str> {
    match config.delivery.as_ref().unwrap_or(&PageDelivery::Composer) {
        PageDelivery::Composer if config.rfa.as_deref().unwrap_or_default().is_empty() => {
            Err("Composer pages must define an RFA")
        }
        PageDelivery::UpstreamProxy { origin, .. } if origin.is_empty() => {
            Err("Proxy pages must define an upstream origin")
        }
        _ => Ok(()),
    }
}

fn validate_page_interaction(
    page_type: &PageType,
    has_interaction: bool,
) -> Result<(), &'static str> {
    match (page_type, has_interaction) {
        (PageType::Rfa, true) => Err("RFA pages must not define interaction config"),
        (PageType::Ifa, false) => Err("IFA pages must define interaction config"),
        _ => Ok(()),
    }
}

fn validate_submit_config(
    method: &str,
    submit: Option<&SubmitServiceData>,
) -> Result<(), &'static str> {
    match (method, submit) {
        ("POST", Some(submit)) if submit.method == "POST" => Ok(()),
        ("POST", Some(_)) => Err("POST page submit config must use POST method"),
        ("POST", None) => Err("POST pages must define submit config"),
        (_, Some(_)) => Err("Only POST pages may define submit config"),
        _ => Ok(()),
    }
}

fn page_config_from_dto(config: &PageConfigDto) -> PageConfig {
    PageConfig {
        path: config.path.clone(),
        method: page_method(config.method.as_deref()),
        page_id: config.page_id.clone(),
        page_type: config.page_type.clone(),
        template: config.template.clone().unwrap_or_default(),
        rfa: config.rfa.clone().unwrap_or_default(),
        delivery: config.delivery.clone().unwrap_or_default(),
        timeout_ms: config.timeout_ms,
        content_type: config
            .content_type
            .clone()
            .unwrap_or_else(|| "text/html; charset=utf-8".into()),
        submit: config.submit.clone(),
        data: config
            .data
            .clone()
            .unwrap_or_default()
            .into_iter()
            .collect(),
        interaction: config.interaction.clone(),
    }
}

fn page_config_to_dto(config: &PageConfig) -> PageConfigDto {
    PageConfigDto {
        path: config.path.clone(),
        method: Some(config.method.clone()),
        page_id: config.page_id.clone(),
        page_type: config.page_type.clone(),
        template: Some(config.template.clone()),
        rfa: Some(config.rfa.clone()),
        delivery: Some(config.delivery.clone()),
        timeout_ms: config.timeout_ms,
        content_type: Some(config.content_type.clone()),
        submit: config.submit.clone(),
        data: Some(config.data.clone().into_iter().collect()),
        interaction: config.interaction.clone(),
    }
}

fn page_method(method: Option<&str>) -> String {
    method.unwrap_or("GET").to_ascii_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfa_page_must_not_define_interaction_config() {
        let page_config = test_page_config(PageType::Rfa, Some(serde_json::json!({})));

        assert_eq!(
            validate_page_config(&page_config),
            Err("RFA pages must not define interaction config")
        );
    }

    #[test]
    fn ifa_page_must_define_interaction_config() {
        let page_config = test_page_config(PageType::Ifa, None);

        assert_eq!(
            validate_page_config(&page_config),
            Err("IFA pages must define interaction config")
        );
    }

    #[test]
    fn dto_without_data_defaults_to_empty_data() {
        let page_config = page_config_from_dto(&PageConfigDto {
            path: "/index.html".to_string(),
            method: None,
            page_id: "landing".to_string(),
            page_type: PageType::Rfa,
            template: Some("landing".to_string()),
            rfa: Some("landing_v1".to_string()),
            delivery: None,
            timeout_ms: 1000,
            content_type: None,
            submit: None,
            data: None,
            interaction: None,
        });

        assert!(page_config.data.is_empty());
        assert_eq!(page_config.method, "GET");
        assert_eq!(page_config.content_type, "text/html; charset=utf-8");
    }

    #[test]
    fn post_page_must_define_submit_config() {
        let mut page_config = test_page_config(PageType::Rfa, None);
        page_config.method = "POST".to_string();

        assert_eq!(
            validate_page_config(&page_config),
            Err("POST pages must define submit config")
        );
    }

    fn test_page_config(page_type: PageType, interaction: Option<serde_json::Value>) -> PageConfig {
        PageConfig {
            path: "/index.html".to_string(),
            method: "GET".to_string(),
            page_id: "landing".to_string(),
            page_type,
            template: "landing".to_string(),
            rfa: "landing_v1".to_string(),
            delivery: PageDelivery::Composer,
            timeout_ms: 1000,
            content_type: "text/html; charset=utf-8".to_string(),
            submit: None,
            data: HashMap::new(),
            interaction,
        }
    }
}
