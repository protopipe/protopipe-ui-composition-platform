use crate::page;

use super::assign::namespace_matches;
use super::config::{ExperimentScope, PageOverrides, RfaOverride};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RfaReplacement {
    pub old: String,
    pub new: String,
    pub namespace: Option<String>,
}

pub(super) fn apply_overrides(
    page_config: &mut page::PageConfig,
    rfa_replacements: &mut Vec<RfaReplacement>,
    experiment_scope: &ExperimentScope,
    overrides: &PageOverrides,
) {
    apply_page_type(page_config, overrides);
    apply_template(page_config, overrides);
    apply_rfa(page_config, rfa_replacements, experiment_scope, overrides);
    apply_delivery(page_config, overrides);
    apply_timeout(page_config, overrides);
    apply_content_type(page_config, overrides);
    apply_data(page_config, overrides);
    apply_interaction(page_config, overrides);
}

fn apply_delivery(page_config: &mut page::PageConfig, overrides: &PageOverrides) {
    if let Some(delivery) = &overrides.delivery {
        page_config.delivery = delivery.clone();
    }
}

fn apply_page_type(page_config: &mut page::PageConfig, overrides: &PageOverrides) {
    if let Some(page_type) = &overrides.page_type {
        page_config.page_type = page_type.clone();
    }
}

fn apply_template(page_config: &mut page::PageConfig, overrides: &PageOverrides) {
    if let Some(template) = &overrides.template {
        page_config.template = template.clone();
    }
}

fn apply_rfa(
    page_config: &mut page::PageConfig,
    rfa_replacements: &mut Vec<RfaReplacement>,
    experiment_scope: &ExperimentScope,
    overrides: &PageOverrides,
) {
    if let Some(rfa) = &overrides.rfa {
        match rfa {
            RfaOverride::Direct(rfa) => page_config.rfa = rfa.clone(),
            RfaOverride::Replace { old, new } => {
                add_rfa_replacement(rfa_replacements, experiment_scope, old, new);
                apply_root_rfa_replacement(page_config, experiment_scope, old, new);
            }
        }
    }
}

fn add_rfa_replacement(
    rfa_replacements: &mut Vec<RfaReplacement>,
    experiment_scope: &ExperimentScope,
    old: &str,
    new: &str,
) {
    rfa_replacements.push(RfaReplacement {
        old: old.to_string(),
        new: new.to_string(),
        namespace: experiment_scope.namespace.clone(),
    });
}

fn apply_root_rfa_replacement(
    page_config: &mut page::PageConfig,
    experiment_scope: &ExperimentScope,
    old: &str,
    new: &str,
) {
    if page_config.rfa == old
        && experiment_scope
            .namespace
            .as_ref()
            .map_or(true, |namespace| {
                namespace_matches(namespace, &page_config.rfa)
            })
    {
        page_config.rfa = new.to_string();
    }
}

fn apply_timeout(page_config: &mut page::PageConfig, overrides: &PageOverrides) {
    if let Some(timeout_ms) = overrides.timeout_ms {
        page_config.timeout_ms = timeout_ms;
    }
}

fn apply_content_type(page_config: &mut page::PageConfig, overrides: &PageOverrides) {
    if let Some(content_type) = &overrides.content_type {
        page_config.content_type = content_type.clone();
    }
}

fn apply_data(page_config: &mut page::PageConfig, overrides: &PageOverrides) {
    if let Some(data) = &overrides.data {
        for (key, value) in data {
            page_config.data.insert(key.clone(), value.clone());
        }
    }
}

fn apply_interaction(page_config: &mut page::PageConfig, overrides: &PageOverrides) {
    if let Some(interaction) = &overrides.interaction {
        page_config.interaction = Some(interaction.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn direct_rfa_override_replaces_root_rfa() {
        let mut page_config = test_page_config("p_landing_v1");
        let mut rfa_replacements = Vec::new();
        let overrides = PageOverrides {
            rfa: Some(RfaOverride::Direct("p_landing_v2".to_string())),
            ..Default::default()
        };

        apply_overrides(
            &mut page_config,
            &mut rfa_replacements,
            &ExperimentScope::default(),
            &overrides,
        );

        assert_eq!(page_config.rfa, "p_landing_v2");
        assert!(rfa_replacements.is_empty());
    }

    #[test]
    fn partial_rfa_override_is_recorded_as_replacement() {
        let mut page_config = test_page_config("p_landing_v1");
        let mut rfa_replacements = Vec::new();
        let overrides = PageOverrides {
            rfa: Some(RfaOverride::Replace {
                old: "a_button_v1".to_string(),
                new: "a_button_v2".to_string(),
            }),
            ..Default::default()
        };

        apply_overrides(
            &mut page_config,
            &mut rfa_replacements,
            &ExperimentScope {
                path: None,
                namespace: Some("p_landing_v1.*".to_string()),
            },
            &overrides,
        );

        assert_eq!(page_config.rfa, "p_landing_v1");
        assert_eq!(rfa_replacements.len(), 1);
        assert_eq!(rfa_replacements[0].old, "a_button_v1");
        assert_eq!(rfa_replacements[0].new, "a_button_v2");
        assert_eq!(
            rfa_replacements[0].namespace.as_deref(),
            Some("p_landing_v1.*")
        );
    }

    #[test]
    fn root_rfa_replacement_respects_namespace_scope() {
        let mut page_config = test_page_config("p_landing_v1");
        let mut rfa_replacements = Vec::new();
        let overrides = PageOverrides {
            rfa: Some(RfaOverride::Replace {
                old: "p_landing_v1".to_string(),
                new: "p_landing_v2".to_string(),
            }),
            ..Default::default()
        };

        apply_overrides(
            &mut page_config,
            &mut rfa_replacements,
            &ExperimentScope {
                path: None,
                namespace: Some("p_other_v1.*".to_string()),
            },
            &overrides,
        );

        assert_eq!(page_config.rfa, "p_landing_v1");
    }

    fn test_page_config(rfa: &str) -> page::PageConfig {
        page::PageConfig {
            path: "/index.html".to_string(),
            method: "GET".to_string(),
            page_id: "page".to_string(),
            page_type: page::PageType::Rfa,
            template: "template".to_string(),
            rfa: rfa.to_string(),
            delivery: page::PageDelivery::Composer,
            timeout_ms: 3000,
            content_type: "text/html; charset=utf-8".to_string(),
            submit: None,
            data: HashMap::new(),
            interaction: None,
        }
    }
}
