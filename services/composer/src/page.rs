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

/// A runtime value sourced from one GET parameter.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetParameterData {
    pub key: String,
}

/// A typed data value for page config.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum DataValue {
    Static(StaticData),
    DynamicRest(DynamicRestData),
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

/// Page configuration with structured data values.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PageConfig {
    pub path: String,
    pub page_id: String,
    #[serde(rename = "type")]
    pub page_type: PageType,
    pub template: String,
    pub rfa: String,
    pub timeout_ms: u64,
    pub content_type: String,
    pub data: HashMap<String, DataValue>,
    pub interaction: Option<serde_json::Value>,
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
    #[serde(rename = "type")]
    pub page_type: PageType,
    pub template: String,
    pub rfa: String,
    pub timeout_ms: u64,
    pub content_type: Option<String>,
    pub data: Option<PageDataDto>,
    pub interaction: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct RFAConfigDto {
    pub id: String,
    pub source: String,
    pub version: String,
}

pub fn resolve_page(state: &AppState, path: &str) -> Option<PageConfig> {
    let pages = state.pages.lock().unwrap();
    resolve_page_from_pages(&pages, path)
}

pub fn request_target(path: &str, query_string: &str) -> String {
    if query_string.is_empty() {
        path.to_string()
    } else {
        format!("{path}?{query_string}")
    }
}

pub fn resolve_rfa(state: &AppState, rfa_id: &str) -> Option<RFAConfig> {
    let rfas = state.rfas.lock().unwrap();
    rfas.get(rfa_id).cloned()
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

    let page_config = PageConfig {
        path: config.path.clone(),
        page_id: config.page_id.clone(),
        page_type: config.page_type.clone(),
        template: config.template.clone(),
        rfa: config.rfa.clone(),
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
    };

    let mut pages = state.pages.lock().unwrap();
    pages.insert(config.path.clone(), page_config);

    log::info!("Registered page: {}", config.path);
    HttpResponse::Created().json(config.into_inner())
}

pub async fn get_pages(state: web::Data<AppState>) -> HttpResponse {
    let pages = state.pages.lock().unwrap();
    let page_list: Vec<PageConfigDto> = pages
        .values()
        .map(|config| PageConfigDto {
            path: config.path.clone(),
            page_id: config.page_id.clone(),
            page_type: config.page_type.clone(),
            template: config.template.clone(),
            rfa: config.rfa.clone(),
            timeout_ms: config.timeout_ms,
            content_type: Some(config.content_type.clone()),
            data: Some(config.data.clone().into_iter().collect()),
            interaction: config.interaction.clone(),
        })
        .collect();

    HttpResponse::Ok().json(page_list)
}

pub async fn reset_config(state: web::Data<AppState>) {
    {
        let mut pages = state.pages.lock().unwrap();
        pages.clear();
    }
    {
        let mut rfas = state.rfas.lock().unwrap();
        rfas.clear();
    }
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

pub fn validate_page_config(config: &PageConfig) -> Result<(), &'static str> {
    validate_page_interaction(&config.page_type, config.interaction.is_some())
}

fn validate_page_config_dto(config: &PageConfigDto) -> Result<(), &'static str> {
    validate_page_interaction(&config.page_type, config.interaction.is_some())
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

fn resolve_page_from_pages(pages: &HashMap<String, PageConfig>, path: &str) -> Option<PageConfig> {
    if let Some(page) = pages.get(path) {
        return Some(page.clone());
    }

    if let Some(page) = resolve_wildcard_page(pages, path) {
        return Some(page);
    }

    let Some(path_without_query) = path.split_once('?').map(|(path, _)| path) else {
        return None;
    };

    if let Some(page) = pages.get(path_without_query) {
        return Some(page.clone());
    }

    resolve_wildcard_page(pages, path_without_query)
}

fn resolve_wildcard_page(pages: &HashMap<String, PageConfig>, path: &str) -> Option<PageConfig> {
    pages
        .iter()
        .filter(|(pattern, _)| pattern.contains('*') && wildcard_matches(pattern, path))
        .max_by(|(left, _), (right, _)| {
            page_pattern_specificity(left).cmp(&page_pattern_specificity(right))
        })
        .map(|(_, page)| page.clone())
}

fn page_pattern_specificity(pattern: &str) -> (usize, usize) {
    let concrete_chars = pattern
        .chars()
        .filter(|character| *character != '*')
        .count();
    let wildcard_count = pattern
        .chars()
        .filter(|character| *character == '*')
        .count();

    (concrete_chars, usize::MAX - wildcard_count)
}

fn wildcard_matches(pattern: &str, value: &str) -> bool {
    if pattern == "*" || pattern == value {
        return true;
    }

    if !pattern.contains('*') {
        return false;
    }

    let parts: Vec<&str> = pattern.split('*').collect();
    let mut rest = value;

    if let Some(first) = parts.first().filter(|first| !first.is_empty()) {
        let Some(stripped) = rest.strip_prefix(first) else {
            return false;
        };
        rest = stripped;
    }

    for part in parts.iter().skip(1).take(parts.len().saturating_sub(2)) {
        if part.is_empty() {
            continue;
        }

        let Some(index) = rest.find(part) else {
            return false;
        };
        rest = &rest[index + part.len()..];
    }

    if let Some(last) = parts.last().filter(|last| !last.is_empty()) {
        rest.ends_with(last)
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_page_matches_generic_path() {
        let pages = HashMap::from([(
            "/my/shop/*".to_string(),
            test_page_config("/my/shop/*", "generic-shop-page"),
        )]);

        let page = resolve_page_from_pages(&pages, "/my/shop/some-category").unwrap();

        assert_eq!(page.page_id, "generic-shop-page");
    }

    #[test]
    fn resolve_page_matches_query_parameter_pattern() {
        let pages = HashMap::from([(
            "/my/shop/search?query=*".to_string(),
            test_page_config("/my/shop/search?query=*", "search-page"),
        )]);

        let page = resolve_page_from_pages(&pages, "/my/shop/search?query=shoes").unwrap();

        assert_eq!(page.page_id, "search-page");
    }

    #[test]
    fn resolve_page_without_query_still_matches_request_with_query() {
        let pages = HashMap::from([(
            "/my/shop/search".to_string(),
            test_page_config("/my/shop/search", "search-page"),
        )]);

        let page = resolve_page_from_pages(&pages, "/my/shop/search?query=shoes").unwrap();

        assert_eq!(page.page_id, "search-page");
    }

    #[test]
    fn query_parameter_pattern_beats_generic_path_pattern() {
        let pages = HashMap::from([
            (
                "/my/shop/*".to_string(),
                test_page_config("/my/shop/*", "generic-shop-page"),
            ),
            (
                "/my/shop/search?query=*".to_string(),
                test_page_config("/my/shop/search?query=*", "search-page"),
            ),
        ]);

        let page = resolve_page_from_pages(&pages, "/my/shop/search?query=shoes").unwrap();

        assert_eq!(page.page_id, "search-page");
    }

    #[test]
    fn resolve_page_prefers_exact_path_over_generic_path() {
        let pages = HashMap::from([
            (
                "/my/shop/*".to_string(),
                test_page_config("/my/shop/*", "generic-shop-page"),
            ),
            (
                "/my/shop/cart.fancy".to_string(),
                test_page_config("/my/shop/cart.fancy", "cart-page"),
            ),
        ]);

        let page = resolve_page_from_pages(&pages, "/my/shop/cart.fancy").unwrap();

        assert_eq!(page.page_id, "cart-page");
    }

    #[test]
    fn resolve_page_prefers_more_specific_generic_path() {
        let pages = HashMap::from([
            (
                "/my/shop/*".to_string(),
                test_page_config("/my/shop/*", "generic-shop-page"),
            ),
            (
                "/my/shop/special-*".to_string(),
                test_page_config("/my/shop/special-*", "special-shop-page"),
            ),
        ]);

        let page = resolve_page_from_pages(&pages, "/my/shop/special-offers").unwrap();

        assert_eq!(page.page_id, "special-shop-page");
    }

    #[test]
    fn wildcard_matches_infix_path_segments() {
        assert!(wildcard_matches(
            "/shop/*/index.html",
            "/shop/sneakers/index.html"
        ));
    }

    fn test_page_config(path: &str, page_id: &str) -> PageConfig {
        PageConfig {
            path: path.to_string(),
            page_id: page_id.to_string(),
            page_type: PageType::Rfa,
            template: "landing".to_string(),
            rfa: "landing_v1".to_string(),
            timeout_ms: 1000,
            content_type: "text/html; charset=utf-8".to_string(),
            data: HashMap::new(),
            interaction: None,
        }
    }
}
