use crate::page;
use crate::AppState;
use actix_web::{
    cookie::{Cookie, SameSite},
    web, HttpRequest, HttpResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const EXPERIMENT_COOKIE_CONSENT: &str = "pp_xa_allowd";

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
    pub page_type: Option<page::PageType>,
    pub template: Option<String>,
    pub rfa: Option<String>,
    pub timeout_ms: Option<u64>,
    pub content_type: Option<String>,
    pub data: Option<HashMap<String, page::DataValue>>,
    pub interaction: Option<serde_json::Value>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct PageOverridesDto {
    #[serde(rename = "type")]
    pub page_type: Option<page::PageType>,
    pub template: Option<String>,
    pub rfa: Option<String>,
    pub timeout_ms: Option<u64>,
    pub content_type: Option<String>,
    pub data: Option<HashMap<String, page::DataValue>>,
    pub interaction: Option<serde_json::Value>,
}

pub struct ResolvedPageConfig {
    pub page_config: page::PageConfig,
    pub assignment_cookie: Option<Cookie<'static>>,
}

struct VariantAssignment<'a> {
    variant: &'a Variant,
    should_set_cookie: bool,
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

        if should_delete_experiment_cookie(req, &cookie_name) {
            assignment_cookie = Some(expire_experiment_cookie(&cookie_name));
            break;
        }

        if let Some(assignment) = determine_variant(experiment, req, &cookie_name) {
            apply_overrides(&mut page_config, &assignment.variant.overrides);

            if assignment.should_set_cookie {
                assignment_cookie = Some(
                    Cookie::build(cookie_name, assignment.variant.id.clone())
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

fn determine_variant<'a>(
    experiment: &'a ExperimentConfig,
    req: &HttpRequest,
    cookie_name: &str,
) -> Option<VariantAssignment<'a>> {
    if !has_experiment_cookie_consent(req) {
        return None;
    }

    if let Some(variant) = req
        .cookie(cookie_name)
        .and_then(|cookie| experiment.variant(cookie.value()))
    {
        return Some(VariantAssignment {
            variant,
            should_set_cookie: false,
        });
    }

    experiment
        .default_variant()
        .map(|variant| VariantAssignment {
            variant,
            should_set_cookie: true,
        })
}

fn has_experiment_cookie_consent(req: &HttpRequest) -> bool {
    req.cookie(EXPERIMENT_COOKIE_CONSENT).is_some()
}

fn should_delete_experiment_cookie(req: &HttpRequest, cookie_name: &str) -> bool {
    !has_experiment_cookie_consent(req) && req.cookie(cookie_name).is_some()
}

fn expire_experiment_cookie(cookie_name: &str) -> Cookie<'static> {
    Cookie::build(cookie_name.to_string(), "")
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(actix_web::cookie::time::Duration::seconds(0))
        .finish()
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

fn apply_overrides(page_config: &mut page::PageConfig, overrides: &PageOverrides) {
    if let Some(page_type) = &overrides.page_type {
        page_config.page_type = page_type.clone();
    }

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

    if let Some(interaction) = &overrides.interaction {
        page_config.interaction = Some(interaction.clone());
    }
}

fn experiment_cookie_name(experiment_id: &str) -> String {
    format!("pp_experiment_{}", experiment_id)
}
