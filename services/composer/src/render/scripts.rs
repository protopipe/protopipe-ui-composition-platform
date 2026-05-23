use deno_core::{error::AnyError, FastString};

use crate::page;

const REGISTRY_JS: &str = include_str!("js/registry.js");
const OUTPUT_JS: &str = include_str!("js/output.js");
const NAMESPACE_JS: &str = include_str!("js/namespace.js");
const REPLACEMENTS_JS: &str = include_str!("js/replacements.js");
const PARTIALS_JS: &str = include_str!("js/partials.js");
const RENDER_JS: &str = include_str!("js/render.js");

pub fn initialize_runtime() -> FastString {
    FastString::from(format!(
        "{REGISTRY_JS}\n{OUTPUT_JS}\n{NAMESPACE_JS}\n{REPLACEMENTS_JS}\n{PARTIALS_JS}\n{RENDER_JS}\n\
         globalThis.protopipeRenderRuntime.initializeRegistry();"
    ))
}

pub fn reset_registry() -> FastString {
    FastString::from("globalThis.protopipeRenderRuntime.resetRegistry();".to_string())
}

pub fn register_rfa(rfa: &page::RFAConfig) -> FastString {
    FastString::from(format!(
        "globalThis.protopipeRenderRuntime.registerRfa({}, {});",
        serde_json::to_string(&rfa.id).unwrap(),
        rfa.source
    ))
}

pub fn render_rfa(
    rfa_id: &str,
    context_json: &str,
    rfa_replacements_json: &str,
) -> Result<FastString, AnyError> {
    let script = format!(
        r#"
(function() {{
    return globalThis.protopipeRenderRuntime.renderRfa({{
        rfaId: {rfa_id},
        contextJson: {context_json:?},
        rfaReplacementsJson: {rfa_replacements_json:?}
    }});
}})()
"#,
        rfa_id = serde_json::to_string(rfa_id)?,
        context_json = context_json,
        rfa_replacements_json = rfa_replacements_json
    );

    Ok(FastString::from(script))
}

#[cfg(test)]
mod tests {
    use deno_core::{serde_v8, v8, JsRuntime, RuntimeOptions};

    use super::*;

    #[test]
    fn javascript_runtime_renders_partial_with_namespace_context() {
        let mut runtime = initialized_runtime();
        register_test_rfa(
            &mut runtime,
            "root",
            "function(context, partials) { return partials.include('child', context); }",
        );
        register_test_rfa(
            &mut runtime,
            "child",
            "function(context) { return context.namespace; }",
        );

        let output = execute_render(&mut runtime, "root", "{}", "[]");

        assert_eq!(output, "root.child");
    }

    #[test]
    fn javascript_runtime_renders_partial_with_indexed_namespace_context() {
        let mut runtime = initialized_runtime();
        register_test_rfa(
            &mut runtime,
            "root",
            "function(context, partials) { return partials.include('child', context, 1); }",
        );
        register_test_rfa(
            &mut runtime,
            "child",
            "function(context) { return context.namespace; }",
        );

        let output = execute_render(&mut runtime, "root", "{}", "[]");

        assert_eq!(output, "root.1.child");
    }

    #[test]
    fn javascript_runtime_applies_namespaced_partial_replacement() {
        let mut runtime = initialized_runtime();
        register_test_rfa(
            &mut runtime,
            "root",
            "function(context, partials) { return partials.include('button', context); }",
        );
        register_test_rfa(&mut runtime, "button", "function() { return 'old'; }");
        register_test_rfa(&mut runtime, "button_new", "function() { return 'new'; }");

        let replacements = r#"[{"old":"button","new":"button_new","namespace":"root.button"}]"#;
        let output = execute_render(&mut runtime, "root", "{}", replacements);

        assert_eq!(output, "new");
    }

    fn initialized_runtime() -> JsRuntime {
        let mut runtime = JsRuntime::new(RuntimeOptions::default());
        runtime
            .execute_script("<test-init>", initialize_runtime())
            .unwrap();
        runtime
    }

    fn register_test_rfa(runtime: &mut JsRuntime, id: &str, source: &str) {
        let rfa = page::RFAConfig {
            id: id.to_string(),
            source: source.to_string(),
            version: "test".to_string(),
        };
        runtime
            .execute_script("<test-rfa-register>", register_rfa(&rfa))
            .unwrap();
    }

    fn execute_render(
        runtime: &mut JsRuntime,
        rfa_id: &str,
        context_json: &str,
        rfa_replacements_json: &str,
    ) -> String {
        let result = runtime
            .execute_script(
                "<test-render>",
                render_rfa(rfa_id, context_json, rfa_replacements_json).unwrap(),
            )
            .unwrap();
        deno_core::scope!(scope, runtime);
        let local = v8::Local::new(scope, result);
        serde_v8::from_v8(scope, local).unwrap()
    }
}
