use crate::{service, AppState};
use reqwest::Method;
use serde_json::Value;
use std::time::Duration;

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
