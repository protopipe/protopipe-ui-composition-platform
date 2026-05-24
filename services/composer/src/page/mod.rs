mod page_config;
mod page_resolver;
mod rfa_config;
mod rfa_resolver;

use actix_web::{web, HttpResponse};

use crate::AppState;

#[allow(unused_imports)]
pub use page_config::{
    validate_page_config, ComposerRoute, DataValue, DynamicRestData, GetParameterData, PageConfig,
    PageConfigDto, PageDataDto, PageDelivery, PageType, PostServiceData, ProxyMarkerReplacement, RedirectData, RestServiceData,
    StaticData, SubmitRouteConfig
};
pub use page_resolver::request_target;
pub use rfa_config::{RFAConfig, RFAConfigDto};

pub fn resolve_route_for_method(
    state: &AppState,
    method: &str,
    path: &str,
) -> Option<ComposerRoute> {
    page_resolver::resolve_page(state, method, path)
}

pub fn resolve_rfa(state: &AppState, rfa_id: &str) -> Option<RFAConfig> {
    rfa_resolver::resolve_rfa(state, rfa_id)
}

pub async fn register_page(
    state: web::Data<AppState>,
    config: web::Json<PageConfigDto>,
) -> HttpResponse {
    page_config::register_page(state, config).await
}

pub async fn get_pages(state: web::Data<AppState>) -> HttpResponse {
    page_config::get_pages(state).await
}

pub async fn register_rfa(
    state: web::Data<AppState>,
    config: web::Json<RFAConfigDto>,
) -> HttpResponse {
    rfa_config::register_rfa(state, config).await
}

pub async fn reset_config(state: web::Data<AppState>) {
    page_config::reset_pages(state.clone()).await;
    rfa_config::reset_rfas(state).await;
}
