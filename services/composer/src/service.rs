use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub id: String,
    pub base_url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceConfigDto {
    pub service_id: String,
    pub base_url: String,
}

pub fn resolve_service(state: &AppState, service_id: &str) -> Option<ServiceConfig> {
    let services = state.services.lock().unwrap();
    services.get(service_id).cloned()
}

pub async fn upsert_service(
    state: web::Data<AppState>,
    config: web::Json<ServiceConfigDto>,
) -> HttpResponse {
    if let Err(message) = validate_service_config_dto(&config) {
        return HttpResponse::BadRequest()
            .content_type("text/plain; charset=utf-8")
            .body(message);
    }

    let service_config = service_config_from_dto(&config);

    let mut services = state.services.lock().unwrap();
    services.insert(service_config.id.clone(), service_config);

    log::info!("Registered service: {}", config.service_id);
    HttpResponse::Created().json(config.into_inner())
}

pub async fn get_services(state: web::Data<AppState>) -> HttpResponse {
    let services = state.services.lock().unwrap();
    let service_list: Vec<ServiceConfigDto> =
        services.values().map(service_config_to_dto).collect();

    HttpResponse::Ok().json(service_list)
}

pub async fn get_service(
    state: web::Data<AppState>,
    service_id: web::Path<String>,
) -> HttpResponse {
    match resolve_service(&state, service_id.as_str()) {
        Some(service) => HttpResponse::Ok().json(service_config_to_dto(&service)),
        None => HttpResponse::NotFound()
            .content_type("text/plain; charset=utf-8")
            .body(format!("Service not found: {}", service_id)),
    }
}

pub async fn delete_service(
    state: web::Data<AppState>,
    service_id: web::Path<String>,
) -> HttpResponse {
    let mut services = state.services.lock().unwrap();

    if services.remove(service_id.as_str()).is_some() {
        HttpResponse::NoContent().finish()
    } else {
        HttpResponse::NotFound()
            .content_type("text/plain; charset=utf-8")
            .body(format!("Service not found: {}", service_id))
    }
}

pub async fn reset_config(state: web::Data<AppState>) {
    let mut services = state.services.lock().unwrap();
    services.clear();
}

fn validate_service_config_dto(config: &ServiceConfigDto) -> Result<(), &'static str> {
    if config.service_id.trim().is_empty() {
        return Err("service_id must not be empty");
    }

    if reqwest::Url::parse(&config.base_url).is_err() {
        return Err("base_url must be an absolute URL");
    }

    Ok(())
}

fn service_config_from_dto(config: &ServiceConfigDto) -> ServiceConfig {
    ServiceConfig {
        id: config.service_id.clone(),
        base_url: trim_trailing_slash(&config.base_url),
    }
}

fn service_config_to_dto(config: &ServiceConfig) -> ServiceConfigDto {
    ServiceConfigDto {
        service_id: config.id.clone(),
        base_url: config.base_url.clone(),
    }
}

fn trim_trailing_slash(value: &str) -> String {
    value.trim_end_matches('/').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_config_requires_service_id() {
        let config = ServiceConfigDto {
            service_id: "".to_string(),
            base_url: "http://wiremock:8080".to_string(),
        };

        assert_eq!(
            validate_service_config_dto(&config),
            Err("service_id must not be empty")
        );
    }

    #[test]
    fn service_config_requires_absolute_base_url() {
        let config = ServiceConfigDto {
            service_id: "catalog".to_string(),
            base_url: "/catalog".to_string(),
        };

        assert_eq!(
            validate_service_config_dto(&config),
            Err("base_url must be an absolute URL")
        );
    }

    #[test]
    fn dto_is_converted_to_service_config() {
        let service_config = service_config_from_dto(&ServiceConfigDto {
            service_id: "catalog".to_string(),
            base_url: "http://wiremock:8080/catalog/".to_string(),
        });

        assert_eq!(service_config.id, "catalog");
        assert_eq!(service_config.base_url, "http://wiremock:8080/catalog");
    }
}
