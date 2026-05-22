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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RfaReplacement {
    pub old: String,
    pub new: String,
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

pub struct ResolvedPageConfig {
    pub page_config: page::PageConfig,
    pub rfa_replacements: Vec<RfaReplacement>,
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
    let request_target = page::request_target(req.path(), req.query_string());
    let mut page_config = page::resolve_page(state, &request_target)?;
    let mut rfa_replacements = Vec::new();
    let mut assignment_cookie = None;

    let experiments = state.experiments.lock().unwrap();
    for experiment in experiments.values() {
        let cookie_name = experiment_cookie_name(&experiment.id);

        if should_delete_experiment_cookie(req, &cookie_name) {
            assignment_cookie = Some(expire_experiment_cookie(&cookie_name));
            break;
        }

        if !experiment_applies_to_request(experiment, req, &page_config) {
            continue;
        }

        if let Some(assignment) = determine_variant(experiment, req, &cookie_name) {
            apply_overrides(
                &mut page_config,
                &mut rfa_replacements,
                &experiment.scope,
                &assignment.variant.overrides,
            );

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
        rfa_replacements,
        assignment_cookie,
    })
}

pub async fn register_experiment(
    state: web::Data<AppState>,
    config: web::Json<ExperimentConfigDto>,
) -> HttpResponse {
    let experiment = ExperimentConfig {
        id: config.experiment_id.clone(),
        scope: config.scope.clone().unwrap_or_default(),
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
            scope: Some(experiment.scope.clone()),
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

fn experiment_applies_to_request(
    experiment: &ExperimentConfig,
    req: &HttpRequest,
    page_config: &page::PageConfig,
) -> bool {
    let path_matches = experiment
        .scope
        .path
        .as_ref()
        .map_or(true, |path| path_matches(path, req.path()));
    let namespace_matches = experiment
        .scope
        .namespace
        .as_ref()
        .map_or(true, |namespace| {
            namespace_can_apply_to_page(namespace, &page_config.rfa)
        });

    path_matches && namespace_matches
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

fn apply_overrides(
    page_config: &mut page::PageConfig,
    rfa_replacements: &mut Vec<RfaReplacement>,
    experiment_scope: &ExperimentScope,
    overrides: &PageOverrides,
) {
    if let Some(page_type) = &overrides.page_type {
        page_config.page_type = page_type.clone();
    }

    if let Some(template) = &overrides.template {
        page_config.template = template.clone();
    }

    if let Some(rfa) = &overrides.rfa {
        match rfa {
            RfaOverride::Direct(rfa) => page_config.rfa = rfa.clone(),
            RfaOverride::Replace { old, new } => {
                rfa_replacements.push(RfaReplacement {
                    old: old.clone(),
                    new: new.clone(),
                    namespace: experiment_scope.namespace.clone(),
                });
                if page_config.rfa == *old
                    && experiment_scope
                        .namespace
                        .as_ref()
                        .map_or(true, |namespace| {
                            namespace_matches(namespace, &page_config.rfa)
                        })
                {
                    page_config.rfa = new.clone();
                }
            }
        }
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

fn namespace_matches(pattern: &str, namespace: &str) -> bool {
    if pattern == namespace {
        return true;
    }

    pattern
        .strip_suffix(".*")
        .is_some_and(|prefix| namespace.starts_with(&format!("{prefix}.")))
}

fn path_matches(pattern: &str, path: &str) -> bool {
    wildcard_matches(pattern, path)
}

fn wildcard_matches(pattern: &str, value: &str) -> bool {
    if pattern == "*" || pattern == value {
        return true;
    }

    if !pattern.contains('*') {
        return false;
    }

    let parts: Vec<&str> = pattern.split('*').collect();
    let mut rest = value;

    if let Some(first) = parts.first().filter(|first| !first.is_empty()) {
        let Some(stripped) = rest.strip_prefix(first) else {
            return false;
        };
        rest = stripped;
    }

    for part in parts.iter().skip(1).take(parts.len().saturating_sub(2)) {
        if part.is_empty() {
            continue;
        }

        let Some(index) = rest.find(part) else {
            return false;
        };
        rest = &rest[index + part.len()..];
    }

    if let Some(last) = parts.last().filter(|last| !last.is_empty()) {
        rest.ends_with(last)
    } else {
        true
    }
}

fn namespace_can_apply_to_page(pattern: &str, root_namespace: &str) -> bool {
    if pattern == root_namespace || pattern.starts_with(&format!("{root_namespace}.")) {
        return true;
    }

    pattern.strip_suffix(".*").is_some_and(|prefix| {
        prefix == root_namespace || prefix.starts_with(&format!("{root_namespace}."))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn experiment_with_scope(path: Option<&str>, namespace: Option<&str>) -> ExperimentConfig {
        ExperimentConfig {
            id: "experiment".to_string(),
            scope: ExperimentScope {
                path: path.map(ToOwned::to_owned),
                namespace: namespace.map(ToOwned::to_owned),
            },
            variants: Vec::new(),
        }
    }

    fn test_page_config(path: &str, rfa: &str) -> page::PageConfig {
        page::PageConfig {
            path: path.to_string(),
            page_id: "page".to_string(),
            page_type: page::PageType::Rfa,
            template: "template".to_string(),
            rfa: rfa.to_string(),
            timeout_ms: 3000,
            content_type: "text/html; charset=utf-8".to_string(),
            data: HashMap::new(),
            interaction: None,
        }
    }

    fn test_request(path: &str) -> HttpRequest {
        actix_web::test::TestRequest::with_uri(path).to_http_request()
    }

    #[test]
    fn experiment_without_scope_applies_to_any_request() {
        let experiment = experiment_with_scope(None, None);
        let page_config = test_page_config("/index.html", "p_landing_v1");
        let request = test_request("/index.html");

        assert!(experiment_applies_to_request(
            &experiment,
            &request,
            &page_config
        ));
    }

    #[test]
    fn experiment_with_matching_path_scope_applies_to_request() {
        let experiment = experiment_with_scope(Some("/index.html"), None);
        let page_config = test_page_config("/index.html", "p_landing_v1");
        let request = test_request("/index.html");

        assert!(experiment_applies_to_request(
            &experiment,
            &request,
            &page_config
        ));
    }

    #[test]
    fn experiment_with_prefix_wildcard_path_scope_does_apply_to_request() {
        let experiment = experiment_with_scope(Some("/experiment/*"), None);
        let page_config = test_page_config("/experiment/index.html", "p_landing_v1");
        let request = test_request("/experiment/index.html");

        assert!(experiment_applies_to_request(
            &experiment,
            &request,
            &page_config
        ));
    }

    #[test]
    fn experiment_with_infix_wildcard_path_scope_does_apply_to_request() {
        let experiment = experiment_with_scope(Some("/shop/*/index.html"), None);
        let page_config = test_page_config("/shop/some/folders/index.html", "p_landing_v1");
        let request = test_request("/shop/some/folders/index.html");

        assert!(experiment_applies_to_request(
            &experiment,
            &request,
            &page_config
        ));
    }

    #[test]
    fn experiment_with_different_path_scope_does_not_apply_to_request() {
        let experiment = experiment_with_scope(Some("/index.html"), None);
        let page_config = test_page_config("/other.html", "p_landing_v1");
        let request = test_request("/other.html");

        assert!(!experiment_applies_to_request(
            &experiment,
            &request,
            &page_config
        ));
    }

    #[test]
    fn experiment_with_namespace_scope_applies_to_matching_root_rfa() {
        let experiment = experiment_with_scope(None, Some("p_landing_v1.*"));
        let page_config = test_page_config("/index.html", "p_landing_v1");
        let request = test_request("/index.html");

        assert!(experiment_applies_to_request(
            &experiment,
            &request,
            &page_config
        ));
    }

    #[test]
    fn experiment_with_namespace_scope_does_not_apply_to_unrelated_root_rfa() {
        let experiment = experiment_with_scope(None, Some("p_landing_v1.*"));
        let page_config = test_page_config("/other.html", "p_other_v1");
        let request = test_request("/other.html");

        assert!(!experiment_applies_to_request(
            &experiment,
            &request,
            &page_config
        ));
    }

    #[test]
    fn experiment_requires_path_and_namespace_scope_to_match() {
        let experiment = experiment_with_scope(Some("/index.html"), Some("p_landing_v1.*"));
        let page_config = test_page_config("/index.html", "p_landing_v1");
        let request = test_request("/index.html");

        assert!(experiment_applies_to_request(
            &experiment,
            &request,
            &page_config
        ));

        let other_path_request = test_request("/other.html");
        assert!(!experiment_applies_to_request(
            &experiment,
            &other_path_request,
            &page_config
        ));

        let other_page_config = test_page_config("/index.html", "p_other_v1");
        assert!(!experiment_applies_to_request(
            &experiment,
            &request,
            &other_page_config
        ));
    }
}
