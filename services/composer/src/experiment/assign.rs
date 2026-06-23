use actix_web::{
    cookie::{Cookie, SameSite},
    HttpRequest,
};
use std::collections::HashMap;

use crate::page;

use super::config::{ExperimentConfig, ExperimentScope, Variant};

const EXPERIMENT_COOKIE_CONSENT: &str = "pp_xa_allowd";

pub(super) struct VariantAssignment<'a> {
    pub(super) variant: &'a Variant,
    pub(super) should_set_cookie: bool,
}

pub(super) fn determine_variant<'a>(
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

pub(super) fn experiment_applies_to_request(
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

pub(super) fn experiment_cookie_name(experiment_id: &str) -> String {
    format!("pp_experiment_{}", experiment_id)
}

pub(super) fn assignment_cookie(cookie_name: &str, variant_id: &str) -> Cookie<'static> {
    Cookie::build(cookie_name.to_string(), variant_id.to_string())
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .finish()
}

pub(super) fn expire_experiment_cookie(cookie_name: &str) -> Cookie<'static> {
    Cookie::build(cookie_name.to_string(), "")
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(actix_web::cookie::time::Duration::seconds(0))
        .finish()
}

pub(super) fn should_delete_experiment_cookie(req: &HttpRequest, cookie_name: &str) -> bool {
    !has_experiment_cookie_consent(req) && req.cookie(cookie_name).is_some()
}

fn has_experiment_cookie_consent(req: &HttpRequest) -> bool {
    req.cookie(EXPERIMENT_COOKIE_CONSENT).is_some()
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

pub(super) fn namespace_matches(pattern: &str, namespace: &str) -> bool {
    if pattern == namespace {
        return true;
    }

    pattern
        .strip_suffix(".*")
        .is_some_and(|prefix| namespace.starts_with(&format!("{prefix}.")))
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
    use super::super::config::{PageOverrides, Variant};
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
            delivery: page::PageDelivery::Composer,
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

    #[test]
    fn determine_variant_uses_existing_cookie_assignment() {
        let experiment = test_experiment_with_variants();
        let cookie_name = experiment_cookie_name(&experiment.id);
        let request = actix_web::test::TestRequest::with_uri("/index.html")
            .cookie(Cookie::new(EXPERIMENT_COOKIE_CONSENT, "yes"))
            .cookie(Cookie::new(cookie_name.clone(), "variant-a"))
            .to_http_request();

        let assignment = determine_variant(&experiment, &request, &cookie_name).unwrap();

        assert_eq!(assignment.variant.id, "variant-a");
        assert!(!assignment.should_set_cookie);
    }

    #[test]
    fn determine_variant_selects_default_when_consent_exists_without_assignment() {
        let experiment = test_experiment_with_variants();
        let cookie_name = experiment_cookie_name(&experiment.id);
        let request = actix_web::test::TestRequest::with_uri("/index.html")
            .cookie(Cookie::new(EXPERIMENT_COOKIE_CONSENT, "yes"))
            .to_http_request();

        let assignment = determine_variant(&experiment, &request, &cookie_name).unwrap();

        assert_eq!(assignment.variant.id, "control");
        assert!(assignment.should_set_cookie);
    }

    #[test]
    fn determine_variant_skips_assignment_without_consent() {
        let experiment = test_experiment_with_variants();
        let cookie_name = experiment_cookie_name(&experiment.id);
        let request = test_request("/index.html");

        assert!(determine_variant(&experiment, &request, &cookie_name).is_none());
    }

    fn test_experiment_with_variants() -> ExperimentConfig {
        ExperimentConfig {
            id: "hero-test".to_string(),
            scope: ExperimentScope::default(),
            variants: vec![
                Variant {
                    id: "control".to_string(),
                    weight: 100,
                    overrides: PageOverrides::default(),
                },
                Variant {
                    id: "variant-a".to_string(),
                    weight: 0,
                    overrides: PageOverrides::default(),
                },
            ],
        }
    }
}
