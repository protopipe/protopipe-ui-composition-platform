use super::request::RequestContext;
use crate::{service, AppState};
use reqwest::Method;
use serde_json::Value;
use std::time::Duration;

pub async fn resolve_rest_service(
    state: &AppState,
    rest_service: &crate::page::RestServiceData,
    request_context: &RequestContext,
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
    let mut request = client.request(method, &url);
    if let Some(query) = rest_service_query(rest_service, request_context) {
        request = request.query(&query);
    }

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

fn rest_service_query(
    rest_service: &crate::page::RestServiceData,
    request_context: &RequestContext,
) -> Option<Vec<(String, String)>> {
    let query_mappings = rest_service.request.as_ref()?.query.as_ref()?;
    let mut query = query_mappings
        .iter()
        .filter_map(|(target_name, mapping)| {
            mapped_service_value(mapping, request_context).map(|value| (target_name.clone(), value))
        })
        .collect::<Vec<_>>();
    query.sort_by(|(left, _), (right, _)| left.cmp(right));

    Some(query)
}

fn mapped_service_value(
    mapping: &crate::page::ServiceValueMapping,
    request_context: &RequestContext,
) -> Option<String> {
    match mapping.from {
        crate::page::ServiceValueSource::Query => request_context.query_value(&mapping.name),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_url_joins_base_url_and_path() {
        assert_eq!(
            service_url("http://wiremock:8080/catalog/", "/products/sku-123"),
            "http://wiremock:8080/catalog/products/sku-123"
        );
    }
}
