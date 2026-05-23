use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RFAConfig {
    pub id: String,
    pub source: String,
    pub version: String,
}

#[derive(Serialize, Deserialize)]
pub struct RFAConfigDto {
    pub id: String,
    pub source: String,
    pub version: String,
}

pub async fn register_rfa(
    state: web::Data<AppState>,
    config: web::Json<RFAConfigDto>,
) -> HttpResponse {
    let rfa_config = rfa_config_from_dto(&config);

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

pub async fn reset_rfas(state: web::Data<AppState>) {
    let mut rfas = state.rfas.lock().unwrap();
    rfas.clear();
}

fn rfa_config_from_dto(config: &RFAConfigDto) -> RFAConfig {
    RFAConfig {
        id: config.id.clone(),
        source: config.source.clone(),
        version: config.version.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dto_is_converted_to_rfa_config() {
        let rfa_config = rfa_config_from_dto(&RFAConfigDto {
            id: "landing_v1".to_string(),
            source: "function(context) { return ''; }".to_string(),
            version: "1".to_string(),
        });

        assert_eq!(rfa_config.id, "landing_v1");
        assert_eq!(rfa_config.version, "1");
    }
}
