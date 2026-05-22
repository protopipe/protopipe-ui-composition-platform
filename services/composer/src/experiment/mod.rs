mod apply;
mod assign;
mod config;

use actix_web::{cookie::Cookie, web, HttpRequest, HttpResponse};

use crate::page;
use crate::AppState;

pub use apply::RfaReplacement;
#[allow(unused_imports)]
pub use config::{
    ExperimentConfig, ExperimentConfigDto, ExperimentScope, PageOverrides, PageOverridesDto,
    RfaOverride, Variant, VariantDto,
};

pub struct ResolvedPageConfig {
    pub page_config: page::PageConfig,
    pub rfa_replacements: Vec<RfaReplacement>,
    pub assignment_cookie: Option<Cookie<'static>>,
}

pub fn resolve_page_config(
    state: &web::Data<AppState>,
    req: &HttpRequest,
) -> Option<ResolvedPageConfig> {
    let request_target = page::request_target(req.path(), req.query_string());
    let mut page_config = page::resolve_page(state, &request_target)?;
    let mut rfa_replacements = Vec::new();
    let mut assignment_cookie = None;

    let experiments = state.experiments.lock().unwrap();
    for experiment in experiments.values() {
        let cookie_name = assign::experiment_cookie_name(&experiment.id);

        if assign::should_delete_experiment_cookie(req, &cookie_name) {
            assignment_cookie = Some(assign::expire_experiment_cookie(&cookie_name));
            break;
        }

        if !assign::experiment_applies_to_request(experiment, req, &page_config) {
            continue;
        }

        if let Some(assignment) = assign::determine_variant(experiment, req, &cookie_name) {
            apply::apply_overrides(
                &mut page_config,
                &mut rfa_replacements,
                &experiment.scope,
                &assignment.variant.overrides,
            );

            if assignment.should_set_cookie {
                assignment_cookie = Some(assign::assignment_cookie(
                    &cookie_name,
                    &assignment.variant.id,
                ));
            }

            break;
        }
    }

    Some(ResolvedPageConfig {
        page_config,
        rfa_replacements,
        assignment_cookie,
    })
}

pub async fn register_experiment(
    state: web::Data<AppState>,
    config: web::Json<ExperimentConfigDto>,
) -> HttpResponse {
    config::register_experiment(state, config).await
}

pub async fn get_experiments(state: web::Data<AppState>) -> HttpResponse {
    config::get_experiments(state).await
}

pub async fn reset_config(state: web::Data<AppState>) {
    config::reset_config(state).await;
}
