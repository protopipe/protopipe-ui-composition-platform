mod post;
mod request;
mod rest;

use crate::AppState;
use crate::DataValue;
use actix_web::HttpRequest;
use futures::future::join_all;
use request::RequestContext;
use serde_json::{Map, Value};
use std::collections::HashMap;

pub use post::resolve_submit_service;

/// Build the renderer context from page data values.
///
/// Static and runtime values are resolved locally. REST service values are
/// resolved concurrently through registered Composer services.
pub async fn build_context(
    state: &AppState,
    data: &HashMap<String, DataValue>,
    req: &HttpRequest,
) -> Value {
    let request_context = RequestContext::from_request(req);
    let values = join_all(
        data.iter()
            .map(|(key, value)| resolve_context_value(state, &request_context, key, value)),
    )
    .await;

    let mut context = Map::new();
    context.extend(values);

    Value::Object(context)
}

async fn resolve_context_value(
    state: &AppState,
    request_context: &RequestContext,
    key: &str,
    value: &DataValue,
) -> (String, Value) {
    let item = match value {
        DataValue::Static(static_data) => static_data.value.clone(),
        DataValue::DynamicRest(dynamic) => dynamic.default.clone(),
        DataValue::RestService(rest_service) => {
            rest::resolve_rest_service(state, rest_service, request_context).await
        }
        DataValue::Url => Value::String(request_context.path().to_string()),
        DataValue::GetParameter(get_parameter) => request_context
            .query_value(&get_parameter.key)
            .map(Value::String)
            .unwrap_or(Value::Null),
    };

    (key.to_string(), item)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_request(uri: &str) -> HttpRequest {
        actix_web::test::TestRequest::with_uri(uri).to_http_request()
    }

    fn test_state() -> AppState {
        AppState {
            pages: std::sync::Mutex::new(HashMap::new()),
            experiments: std::sync::Mutex::new(HashMap::new()),
            rfas: std::sync::Mutex::new(HashMap::new()),
            services: std::sync::Mutex::new(HashMap::new()),
            render_pool: crate::render::RenderPool::new(1),
        }
    }

    #[actix_rt::test]
    async fn build_context_injects_declared_url_without_query_string() {
        let mut data = HashMap::new();
        data.insert("url".to_string(), DataValue::Url);
        let request = test_request("/index.html?message=Hello");
        let state = test_state();

        let context = build_context(&state, &data, &request).await;

        assert_eq!(
            context,
            serde_json::json!({
                "url": "/index.html"
            })
        );
    }

    #[actix_rt::test]
    async fn build_context_injects_declared_get_parameter() {
        let mut data = HashMap::new();
        data.insert(
            "getMessage".to_string(),
            DataValue::GetParameter(crate::page::GetParameterData {
                key: "message".to_string(),
            }),
        );
        let request = test_request("/index.html?message=Hello");
        let state = test_state();

        let context = build_context(&state, &data, &request).await;

        assert_eq!(
            context,
            serde_json::json!({
                "getMessage": "Hello"
            })
        );
    }

    #[actix_rt::test]
    async fn build_context_uses_null_for_missing_get_parameter() {
        let mut data = HashMap::new();
        data.insert(
            "getMessage".to_string(),
            DataValue::GetParameter(crate::page::GetParameterData {
                key: "message".to_string(),
            }),
        );
        let request = test_request("/index.html");
        let state = test_state();

        let context = build_context(&state, &data, &request).await;

        assert_eq!(
            context,
            serde_json::json!({
                "getMessage": null
            })
        );
    }

    #[actix_rt::test]
    async fn build_context_uses_error_default_for_missing_rest_service() {
        let mut data = HashMap::new();
        data.insert(
            "product".to_string(),
            DataValue::RestService(crate::page::RestServiceData {
                service: "catalog".to_string(),
                path: "/products/sku-123".to_string(),
                method: Some("GET".to_string()),
                timeout_ms: Some(250),
                request: None,
                error_default: Some(serde_json::json!({
                    "name": "Unknown product"
                })),
            }),
        );
        let request = test_request("/product.html");
        let state = test_state();

        let context = build_context(&state, &data, &request).await;

        assert_eq!(
            context,
            serde_json::json!({
                "product": {
                    "name": "Unknown product"
                }
            })
        );
    }
}
