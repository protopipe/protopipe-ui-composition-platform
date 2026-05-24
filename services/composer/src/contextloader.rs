use crate::DataValue;
use crate::{service, AppState};
use actix_web::HttpRequest;
use futures::future::join_all;
use reqwest::Method;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::time::Duration;

/// Build the renderer context from page data values.
///
/// Static and runtime values are resolved locally. REST service values are
/// resolved concurrently through registered Composer services.
pub async fn build_context(
    state: &AppState,
    data: &HashMap<String, DataValue>,
    req: &HttpRequest,
) -> Value {
    let query_params = query_params(req.query_string());
    let values = join_all(
        data.iter()
            .map(|(key, value)| resolve_context_value(state, req, &query_params, key, value)),
    )
    .await;

    let mut context = Map::new();
    context.extend(values);

    Value::Object(context)
}

async fn resolve_context_value(
    state: &AppState,
    req: &HttpRequest,
    query_params: &Value,
    key: &str,
    value: &DataValue,
) -> (String, Value) {
    let item = match value {
        DataValue::Static(static_data) => static_data.value.clone(),
        DataValue::DynamicRest(dynamic) => dynamic.default.clone(),
        DataValue::RestService(rest_service) => resolve_rest_service(state, rest_service).await,
        DataValue::Url => Value::String(req.path().to_string()),
        DataValue::GetParameter(get_parameter) => query_params
            .get(&get_parameter.key)
            .cloned()
            .unwrap_or(Value::Null),
    };

    (key.to_string(), item)
}

async fn resolve_rest_service(
    state: &AppState,
    rest_service: &crate::page::RestServiceData,
) -> Value {
    let Some(service_config) = service::resolve_service(state, &rest_service.service) else {
        log::warn!("REST service not registered: {}", rest_service.service);
        return rest_error_default(rest_service);
    };
    let Ok(method) = rest_method(rest_service) else {
        log::warn!(
            "Unsupported REST method for service {}: {:?}",
            rest_service.service,
            rest_service.method
        );
        return rest_error_default(rest_service);
    };
    let url = service_url(&service_config.base_url, &rest_service.path);
    let timeout = Duration::from_millis(rest_service.timeout_ms.unwrap_or(1000));
    let client = reqwest::Client::new();
    let request = client.request(method, &url);

    match tokio::time::timeout(timeout, request.send()).await {
        Ok(Ok(response)) if response.status().is_success() => {
            response.json::<Value>().await.unwrap_or_else(|error| {
                log::warn!(
                    "REST service response was not valid JSON at {}: {}",
                    url,
                    error
                );
                rest_error_default(rest_service)
            })
        }
        Ok(Ok(response)) => {
            log::warn!(
                "REST service returned non-success status at {}: {}",
                url,
                response.status()
            );
            rest_error_default(rest_service)
        }
        Ok(Err(error)) => {
            log::warn!("REST service request failed at {}: {}", url, error);
            rest_error_default(rest_service)
        }
        Err(_) => {
            log::warn!("REST service request timed out at {}", url);
            rest_error_default(rest_service)
        }
    }
}

pub async fn resolve_submit_service(
    state: &AppState,
    post_service: &crate::page::PostServiceData,
    body: &[u8],
) -> Value {
    let Some(service_config) = service::resolve_service(state, &post_service.service) else {
        log::warn!("POST service not registered: {}", post_service.service);
        return Value::Null;
    };
    let url = service_url(&service_config.base_url, &post_service.path);
    let timeout = Duration::from_millis(post_service.timeout_ms.unwrap_or(1000));
    let client = reqwest::Client::new();
    let request = client
        .request(Method::POST, &url)
        .header(
            reqwest::header::CONTENT_TYPE,
            post_service.content_type.clone(),
        )
        .body(body.to_vec());

    match tokio::time::timeout(timeout, request.send()).await {
        Ok(Ok(response)) if response.status().is_success() => {
            response.json::<Value>().await.unwrap_or_else(|error| {
                log::warn!(
                    "POST service response was not valid JSON at {}: {}",
                    url,
                    error
                );
                Value::Null
            })
        }
        Ok(Ok(response)) => {
            log::warn!(
                "POST service returned non-success status at {}: {}",
                url,
                response.status()
            );
            Value::Null
        }
        Ok(Err(error)) => {
            log::warn!("POST service request failed at {}: {}", url, error);
            Value::Null
        }
        Err(_) => {
            log::warn!("POST service request timed out at {}", url);
            Value::Null
        }
    }
}

fn rest_method(rest_service: &crate::page::RestServiceData) -> Result<Method, ()> {
    match rest_service.method.as_deref().unwrap_or("GET") {
        "GET" => Ok(Method::GET),
        "POST" => Ok(Method::POST),
        "PUT" => Ok(Method::PUT),
        "DELETE" => Ok(Method::DELETE),
        "PATCH" => Ok(Method::PATCH),
        _ => Err(()),
    }
}

fn service_url(base_url: &str, path: &str) -> String {
    format!(
        "{}{}",
        base_url.trim_end_matches('/'),
        ensure_leading_slash(path)
    )
}

fn ensure_leading_slash(path: &str) -> String {
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    }
}

fn rest_error_default(rest_service: &crate::page::RestServiceData) -> Value {
    rest_service.error_default.clone().unwrap_or(Value::Null)
}

fn query_params(query_string: &str) -> Value {
    let mut params = Map::new();

    for pair in query_string.split('&').filter(|pair| !pair.is_empty()) {
        let mut parts = pair.splitn(2, '=');
        let key = percent_decode(parts.next().unwrap_or_default());
        let value = percent_decode(parts.next().unwrap_or_default());

        if key.is_empty() {
            continue;
        }

        match params.get_mut(&key) {
            Some(Value::Array(values)) => values.push(Value::String(value)),
            Some(existing) => {
                let previous = existing.clone();
                *existing = Value::Array(vec![previous, Value::String(value)]);
            }
            None => {
                params.insert(key, Value::String(value));
            }
        }
    }

    Value::Object(params)
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                output.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                if let (Some(high), Some(low)) =
                    (hex_value(bytes[index + 1]), hex_value(bytes[index + 2]))
                {
                    output.push(high * 16 + low);
                    index += 3;
                } else {
                    output.push(bytes[index]);
                    index += 1;
                }
            }
            byte => {
                output.push(byte);
                index += 1;
            }
        }
    }

    String::from_utf8_lossy(&output).into_owned()
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_request(uri: &str) -> HttpRequest {
        actix_web::test::TestRequest::with_uri(uri).to_http_request()
    }

    #[test]
    fn query_params_returns_single_values_as_strings() {
        let params = query_params("message=Hello&currency=EUR");

        assert_eq!(
            params,
            serde_json::json!({
                "message": "Hello",
                "currency": "EUR"
            })
        );
    }

    #[test]
    fn query_params_returns_repeated_values_as_arrays() {
        let params = query_params("tag=red&tag=blue");

        assert_eq!(
            params,
            serde_json::json!({
                "tag": ["red", "blue"]
            })
        );
    }

    #[test]
    fn query_params_decodes_percent_encoding_and_plus_spaces() {
        let params = query_params("message=Hello+World%21");

        assert_eq!(
            params,
            serde_json::json!({
                "message": "Hello World!"
            })
        );
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

    #[test]
    fn service_url_joins_base_url_and_path() {
        assert_eq!(
            service_url("http://wiremock:8080/catalog/", "/products/sku-123"),
            "http://wiremock:8080/catalog/products/sku-123"
        );
    }
}
