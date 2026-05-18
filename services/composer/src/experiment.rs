use crate::page;
use crate::AppState;
use actix_web::{
    cookie::{Cookie, SameSite},
    web, HttpRequest, HttpResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone)]
pub struct ExperimentConfig {
    pub id: String,
    pub variants: Vec<Variant>,
}

#[derive(Serialize, Deserialize)]
pub struct ExperimentConfigDto {
    pub experiment_id: String,
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
    pub template: Option<String>,
    pub rfa: Option<String>,
    pub timeout_ms: Option<u64>,
    pub content_type: Option<String>,
    pub data: Option<HashMap<String, page::DataValue>>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct PageOverridesDto {
    pub template: Option<String>,
    pub rfa: Option<String>,
    pub timeout_ms: Option<u64>,
    pub content_type: Option<String>,
    pub data: Option<HashMap<String, page::DataValue>>,
}

pub struct ResolvedPageConfig {
    pub page_config: page::PageConfig,
    pub assignment_cookie: Option<Cookie<'static>>,
}

pub fn resolve_page_config(
    state: &web::Data<AppState>,
    req: &HttpRequest,
) -> Option<ResolvedPageConfig> {
    let path = req.path();
    let mut page_config = page::resolve_page(state, path)?;
    let mut assignment_cookie = None;

    let experiments = state.experiments.lock().unwrap();
    for experiment in experiments.values() {
        let cookie_name = experiment_cookie_name(&experiment.id);
        let variant = req
            .cookie(&cookie_name)
            .and_then(|cookie| experiment.variant(cookie.value()))
            .or_else(|| experiment.default_variant());

        if let Some(variant) = variant {
            apply_overrides(&mut page_config, &variant.overrides);

            if req.cookie(&cookie_name).is_none() {
                assignment_cookie = Some(
                    Cookie::build(cookie_name, variant.id.clone())
                        .path("/")
                        .http_only(true)
                        .same_site(SameSite::Lax)
                        .finish(),
                );
            }

            break;
        }
    }

    Some(ResolvedPageConfig {
        page_config,
        assignment_cookie,
    })
}

pub async fn register_experiment(
    state: web::Data<AppState>,
    config: web::Json<ExperimentConfigDto>,
) -> HttpResponse {
    let experiment = ExperimentConfig {
        id: config.experiment_id.clone(),
        variants: config
            .variants
            .iter()
            .map(|variant| Variant {
                id: variant.id.clone(),
                weight: variant.weight,
                overrides: variant.overrides.clone().unwrap_or_default().into(),
            })
            .collect(),
    };

    let mut experiments = state.experiments.lock().unwrap();
    experiments.insert(experiment.id.clone(), experiment);

    log::info!("Registered experiment: {}", config.experiment_id);
    HttpResponse::Created().json(config.into_inner())
}

pub async fn get_experiments(state: web::Data<AppState>) -> HttpResponse {
    let experiments = state.experiments.lock().unwrap();
    let experiment_list: Vec<ExperimentConfigDto> = experiments
        .values()
        .map(|experiment| ExperimentConfigDto {
            experiment_id: experiment.id.clone(),
            variants: experiment
                .variants
                .iter()
                .map(|variant| VariantDto {
                    id: variant.id.clone(),
                    weight: variant.weight,
                    overrides: Some(variant.overrides.clone().into()),
                })
                .collect(),
        })
        .collect();

    HttpResponse::Ok().json(experiment_list)
}

pub async fn reset_config(state: web::Data<AppState>) {
    let mut experiments = state.experiments.lock().unwrap();
    experiments.clear();
}

impl ExperimentConfig {
    fn variant(&self, variant_id: &str) -> Option<&Variant> {
        self.variants
            .iter()
            .find(|variant| variant.id == variant_id)
    }

    fn default_variant(&self) -> Option<&Variant> {
        self.variants.iter().find(|variant| variant.weight > 0)
    }
}

impl From<PageOverridesDto> for PageOverrides {
    fn from(value: PageOverridesDto) -> Self {
        Self {
            template: value.template,
            rfa: value.rfa,
            timeout_ms: value.timeout_ms,
            content_type: value.content_type,
            data: value.data,
        }
    }
}

impl From<PageOverrides> for PageOverridesDto {
    fn from(value: PageOverrides) -> Self {
        Self {
            template: value.template,
            rfa: value.rfa,
            timeout_ms: value.timeout_ms,
            content_type: value.content_type,
            data: value.data,
        }
    }
}

fn apply_overrides(page_config: &mut page::PageConfig, overrides: &PageOverrides) {
    if let Some(template) = &overrides.template {
        page_config.template = template.clone();
    }

    if let Some(rfa) = &overrides.rfa {
        page_config.rfa = rfa.clone();
    }

    if let Some(timeout_ms) = overrides.timeout_ms {
        page_config.timeout_ms = timeout_ms;
    }

    if let Some(content_type) = &overrides.content_type {
        page_config.content_type = content_type.clone();
    }

    if let Some(data) = &overrides.data {
        for (key, value) in data {
            page_config.data.insert(key.clone(), value.clone());
        }
    }
}

fn experiment_cookie_name(experiment_id: &str) -> String {
    format!("experiment_{}", experiment_id)
}
