use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::page;
use crate::AppState;

#[derive(Clone)]
pub struct ExperimentConfig {
    pub id: String,
    pub scope: ExperimentScope,
    pub variants: Vec<Variant>,
}

#[derive(Serialize, Deserialize)]
pub struct ExperimentConfigDto {
    pub experiment_id: String,
    pub scope: Option<ExperimentScope>,
    pub variants: Vec<VariantDto>,
}

#[derive(Serialize, Deserialize)]
pub struct VariantDto {
    pub id: String,
    pub weight: u32,
    pub overrides: Option<PageOverridesDto>,
}

#[derive(Clone)]
pub struct Variant {
    pub id: String,
    pub weight: u32,
    pub overrides: PageOverrides,
}

#[derive(Clone, Default)]
pub struct PageOverrides {
    pub page_type: Option<page::PageType>,
    pub template: Option<String>,
    pub rfa: Option<RfaOverride>,
    pub timeout_ms: Option<u64>,
    pub content_type: Option<String>,
    pub data: Option<HashMap<String, page::DataValue>>,
    pub interaction: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RfaOverride {
    Direct(String),
    Replace { old: String, new: String },
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct ExperimentScope {
    pub path: Option<String>,
    pub namespace: Option<String>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct PageOverridesDto {
    #[serde(rename = "type")]
    pub page_type: Option<page::PageType>,
    pub template: Option<String>,
    pub rfa: Option<RfaOverride>,
    pub timeout_ms: Option<u64>,
    pub content_type: Option<String>,
    pub data: Option<HashMap<String, page::DataValue>>,
    pub interaction: Option<serde_json::Value>,
}

impl ExperimentConfig {
    pub(super) fn variant(&self, variant_id: &str) -> Option<&Variant> {
        self.variants
            .iter()
            .find(|variant| variant.id == variant_id)
    }

    pub(super) fn default_variant(&self) -> Option<&Variant> {
        self.variants.iter().find(|variant| variant.weight > 0)
    }
}

pub async fn register_experiment(
    state: web::Data<AppState>,
    config: web::Json<ExperimentConfigDto>,
) -> HttpResponse {
    let experiment = experiment_config_from_dto(&config);

    let mut experiments = state.experiments.lock().unwrap();
    experiments.insert(experiment.id.clone(), experiment);

    log::info!("Registered experiment: {}", config.experiment_id);
    HttpResponse::Created().json(config.into_inner())
}

pub async fn get_experiments(state: web::Data<AppState>) -> HttpResponse {
    let experiments = state.experiments.lock().unwrap();
    let experiment_list: Vec<ExperimentConfigDto> =
        experiments.values().map(experiment_config_to_dto).collect();

    HttpResponse::Ok().json(experiment_list)
}

pub async fn reset_config(state: web::Data<AppState>) {
    let mut experiments = state.experiments.lock().unwrap();
    experiments.clear();
}

fn experiment_config_from_dto(config: &ExperimentConfigDto) -> ExperimentConfig {
    ExperimentConfig {
        id: config.experiment_id.clone(),
        scope: config.scope.clone().unwrap_or_default(),
        variants: config.variants.iter().map(variant_from_dto).collect(),
    }
}

fn experiment_config_to_dto(experiment: &ExperimentConfig) -> ExperimentConfigDto {
    ExperimentConfigDto {
        experiment_id: experiment.id.clone(),
        scope: Some(experiment.scope.clone()),
        variants: experiment.variants.iter().map(variant_to_dto).collect(),
    }
}

fn variant_from_dto(variant: &VariantDto) -> Variant {
    Variant {
        id: variant.id.clone(),
        weight: variant.weight,
        overrides: variant.overrides.clone().unwrap_or_default().into(),
    }
}

fn variant_to_dto(variant: &Variant) -> VariantDto {
    VariantDto {
        id: variant.id.clone(),
        weight: variant.weight,
        overrides: Some(variant.overrides.clone().into()),
    }
}

impl From<PageOverridesDto> for PageOverrides {
    fn from(value: PageOverridesDto) -> Self {
        Self {
            page_type: value.page_type,
            template: value.template,
            rfa: value.rfa,
            timeout_ms: value.timeout_ms,
            content_type: value.content_type,
            data: value.data,
            interaction: value.interaction,
        }
    }
}

impl From<PageOverrides> for PageOverridesDto {
    fn from(value: PageOverrides) -> Self {
        Self {
            page_type: value.page_type,
            template: value.template,
            rfa: value.rfa,
            timeout_ms: value.timeout_ms,
            content_type: value.content_type,
            data: value.data,
            interaction: value.interaction,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dto_without_scope_defaults_to_unscoped_experiment() {
        let experiment = experiment_config_from_dto(&ExperimentConfigDto {
            experiment_id: "hero-test".to_string(),
            scope: None,
            variants: Vec::new(),
        });

        assert_eq!(experiment.id, "hero-test");
        assert!(experiment.scope.path.is_none());
        assert!(experiment.scope.namespace.is_none());
    }

    #[test]
    fn experiment_returns_cookie_variant_when_present() {
        let experiment = ExperimentConfig {
            id: "hero-test".to_string(),
            scope: ExperimentScope::default(),
            variants: vec![test_variant("control", 100), test_variant("variant-a", 0)],
        };

        assert_eq!(experiment.variant("variant-a").unwrap().id, "variant-a");
    }

    #[test]
    fn experiment_returns_first_weighted_variant_as_default() {
        let experiment = ExperimentConfig {
            id: "hero-test".to_string(),
            scope: ExperimentScope::default(),
            variants: vec![test_variant("disabled", 0), test_variant("control", 100)],
        };

        assert_eq!(experiment.default_variant().unwrap().id, "control");
    }

    fn test_variant(id: &str, weight: u32) -> Variant {
        Variant {
            id: id.to_string(),
            weight,
            overrides: PageOverrides::default(),
        }
    }
}
