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
    pub request: Option<RestServiceRequest>,
    pub error_default: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RestServiceRequest {
    pub query: Option<HashMap<String, ServiceValueMapping>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceValueMapping {
    pub from: ServiceValueSource,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ServiceValueSource {
    Query,
}

/// A POST service call used to process a submitted request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostServiceData {
    pub service: String,
    pub path: String,
    pub content_type: String,
    pub timeout_ms: Option<u64>,
    pub redirect: RedirectData,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RedirectData {
    pub path: String,
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

/// A GET route that renders a page through an RFA or IFA configuration.
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
    pub data: HashMap<String, DataValue>,
    pub interaction: Option<serde_json::Value>,
}

/// A POST route that processes a user intent and redirects to a GET result page.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmitRouteConfig {
    pub path: String,
    pub method: String,
    pub page_id: String,
    pub timeout_ms: u64,
    pub post_service: PostServiceData,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "route_type", rename_all = "kebab-case")]
pub enum ComposerRoute {
    Page(PageConfig),
    Submit(SubmitRouteConfig),
}

impl ComposerRoute {
    pub fn path(&self) -> &str {
        match self {
            ComposerRoute::Page(config) => &config.path,
            ComposerRoute::Submit(config) => &config.path,
        }
    }

    pub fn method(&self) -> &str {
        match self {
            ComposerRoute::Page(config) => &config.method,
            ComposerRoute::Submit(config) => &config.method,
        }
    }
}

pub type PageDataDto = HashMap<String, DataValue>;

#[derive(Serialize, Deserialize)]
pub struct PageConfigDto {
    pub path: String,
    pub method: Option<String>,
    pub page_id: String,
    #[serde(rename = "type")]
    pub page_type: Option<PageType>,
    pub template: Option<String>,
    pub rfa: Option<String>,
    pub delivery: Option<PageDelivery>,
    pub timeout_ms: u64,
    pub content_type: Option<String>,
    #[serde(rename = "postService")]
    pub post_service: Option<PostServiceData>,
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

    let route_config = route_config_from_dto(&config);

    let mut routes = state.pages.lock().unwrap();
    routes.insert(
        route_key(route_config.method(), route_config.path()),
        route_config,
    );

    log::info!("Registered page route: {}", config.path);
    HttpResponse::Created().json(config.into_inner())
}

pub async fn get_pages(state: web::Data<AppState>) -> HttpResponse {
    let routes = state.pages.lock().unwrap();
    let page_list: Vec<PageConfigDto> = routes.values().map(route_config_to_dto).collect();

    HttpResponse::Ok().json(page_list)
}

pub async fn reset_pages(state: web::Data<AppState>) {
    let mut routes = state.pages.lock().unwrap();
    routes.clear();
}

pub fn validate_page_config(config: &PageConfig) -> Result<(), &'static str> {
    validate_page_delivery(config)?;
    validate_page_interaction(&config.page_type, config.interaction.is_some())
}


fn validate_page_config_dto(config: &PageConfigDto) -> Result<(), &'static str> {
    match page_method(config.method.as_deref()).as_str() {
        "POST" => validate_submit_route_dto(config),
        "GET" => validate_page_route_dto(config),
        _ => Err("Only GET and POST page methods are supported"),
    }
}

fn validate_page_route_dto(config: &PageConfigDto) -> Result<(), &'static str> {
    if config.post_service.is_some() {
        return Err("GET page routes must not define postService");
    }

    let page_type = config
        .page_type
        .as_ref()
        .ok_or("GET page routes must define type")?;

    if config.delivery.is_none() {
        if config.template.is_none() {
            return Err("GET page routes must define either template or delivery");
        }

        if config.rfa.is_none() {
            return Err("GET page routes must define rfa");
        }
    }

    validate_page_interaction(page_type, config.interaction.is_some())
}

fn validate_submit_route_dto(config: &PageConfigDto) -> Result<(), &'static str> {
    if config.post_service.is_none() {
        return Err("POST submit routes must define postService");
    }

    if config.page_type.is_some()
        || config.template.is_some()
        || config.rfa.is_some()
        || config.data.is_some()
        || config.interaction.is_some()
        || config.content_type.is_some()
    {
        return Err("POST submit routes must not define rendering config");
    }

    Ok(())
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

fn route_config_from_dto(config: &PageConfigDto) -> ComposerRoute {
    match page_method(config.method.as_deref()).as_str() {
        "POST" => ComposerRoute::Submit(submit_route_from_dto(config)),
        _ => ComposerRoute::Page(page_config_from_dto(config)),
    }
}

fn page_config_from_dto(config: &PageConfigDto) -> PageConfig {
    PageConfig {
        path: config.path.clone(),
        method: "GET".to_string(),
        page_id: config.page_id.clone(),
        page_type: config
            .page_type
            .clone()
            .expect("GET page routes must define type"),
        template: config
            .template
            .clone()
            .unwrap_or_default(),
        delivery: config.delivery.clone().unwrap_or_default(),    
        rfa: config.rfa.clone().unwrap_or_default(),
        timeout_ms: config.timeout_ms,
        content_type: config
            .content_type
            .clone()
            .unwrap_or_else(|| "text/html; charset=utf-8".into()),
        data: config
            .data
            .clone()
            .unwrap_or_default()
            .into_iter()
            .collect(),
        interaction: config.interaction.clone(),
    }
}

fn submit_route_from_dto(config: &PageConfigDto) -> SubmitRouteConfig {
    SubmitRouteConfig {
        path: config.path.clone(),
        method: "POST".to_string(),
        page_id: config.page_id.clone(),
        timeout_ms: config.timeout_ms,
        post_service: config
            .post_service
            .clone()
            .expect("POST submit routes must define postService"),
    }
}

fn route_config_to_dto(route: &ComposerRoute) -> PageConfigDto {
    match route {
        ComposerRoute::Page(config) => page_config_to_dto(config),
        ComposerRoute::Submit(config) => submit_route_to_dto(config),
    }
}

fn page_config_to_dto(config: &PageConfig) -> PageConfigDto {
    PageConfigDto {
        path: config.path.clone(),
        method: Some(config.method.clone()),
        page_id: config.page_id.clone(),
        page_type: Some(config.page_type.clone()),
        template: Some(config.template.clone()),
        rfa: Some(config.rfa.clone()),
        delivery: Some(config.delivery.clone()),
        timeout_ms: config.timeout_ms,
        content_type: Some(config.content_type.clone()),
        post_service: None,
        data: Some(config.data.clone().into_iter().collect()),
        interaction: config.interaction.clone(),
    }
}

fn submit_route_to_dto(config: &SubmitRouteConfig) -> PageConfigDto {
    PageConfigDto {
        path: config.path.clone(),
        method: Some(config.method.clone()),
        page_id: config.page_id.clone(),
        page_type: None,
        template: None,
        delivery: None,
        rfa: None,
        timeout_ms: config.timeout_ms,
        content_type: None,
        post_service: Some(config.post_service.clone()),
        data: None,
        interaction: None,
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
            page_type: Some(PageType::Rfa),
            template: Some("landing".to_string()),
            rfa: Some("landing_v1".to_string()),
            delivery: None,
            timeout_ms: 1000,
            content_type: None,
            post_service: None,
            data: None,
            interaction: None,
        });

        assert!(page_config.data.is_empty());
        assert_eq!(page_config.method, "GET");
        assert_eq!(page_config.content_type, "text/html; charset=utf-8");
    }

    #[test]
    fn post_route_must_define_post_service() {
        assert_eq!(
            validate_page_config_dto(&PageConfigDto {
                path: "/contact.html".to_string(),
                method: Some("POST".to_string()),
                page_id: "contact-submit".to_string(),
                page_type: None,
                template: None,
                delivery: None,
                rfa: None,
                timeout_ms: 1000,
                content_type: None,
                post_service: None,
                data: None,
                interaction: None,
            }),
            Err("POST submit routes must define postService")
        );
    }

    #[test]
    fn post_route_must_not_define_rendering_config() {
        assert_eq!(
            validate_page_config_dto(&PageConfigDto {
                path: "/contact.html".to_string(),
                method: Some("POST".to_string()),
                page_id: "contact-submit".to_string(),
                page_type: Some(PageType::Rfa),
                template: None,
                delivery: None,
                rfa: None,
                timeout_ms: 1000,
                content_type: None,
                post_service: Some(test_post_service()),
                data: None,
                interaction: None,
            }),
            Err("POST submit routes must not define rendering config")
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
            data: HashMap::new(),
            interaction,
        }
    }

    fn test_post_service() -> PostServiceData {
        PostServiceData {
            service: "contact".to_string(),
            path: "/contact-requests".to_string(),
            content_type: "application/x-www-form-urlencoded".to_string(),
            timeout_ms: Some(500),
            redirect: RedirectData {
                path: "/contact/received.html".to_string(),
            },
        }
    }
}
