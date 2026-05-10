use crate::{contextloader, AppState};
use actix_web::{web, HttpRequest, HttpResponse};
use deno_core::{error::AnyError, JsRuntime, RuntimeOptions, serde_v8, v8};

fn execute_rfa(source: &str, context: &serde_json::Value) -> Result<String, AnyError> {
    let mut runtime = JsRuntime::new(RuntimeOptions::default());
    let context_json = serde_json::to_string(context)?;

    let script = format!(
        r#"const render = {source};
const context = JSON.parse({context_json:?});
const output = render(context);
if (typeof output === 'string') output;
else JSON.stringify(output);
"#,
        source = source,
        context_json = context_json
    );

    let result = runtime.execute_script(
        "<rfa>", 
        deno_core::FastString::from(script)
    )?;
    let scope = &mut runtime.handle_scope();
    let local = v8::Local::new(scope, result);
    let output: String = serde_v8::from_v8(scope, local)?;
    Ok(output)
}

pub async fn render_page(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> HttpResponse {
    let path = req.path();

    log::debug!("Render request: {}", path);

    let pages = state.pages.lock().unwrap();

    match pages.get(path) {
        Some(page_config) => {
            let rfas = state.rfas.lock().unwrap();
            let rfa = match rfas.get(&page_config.rfa) {
                Some(rfa) => rfa,
                None => {
                    log::error!("RFA not found: {}", page_config.rfa);
                    return HttpResponse::InternalServerError()
                        .content_type("text/plain; charset=utf-8")
                        .body(format!("RFA not found: {}", page_config.rfa));
                }
            };

            let context = contextloader::build_context(&page_config.data);
            let rendered = match execute_rfa(&rfa.source, &context) {
                Ok(output) => output,
                Err(err) => {
                    log::error!("RFA execution failed: {}", err);
                    return HttpResponse::InternalServerError()
                        .content_type("text/plain; charset=utf-8")
                        .body(format!("RFA execution failed: {}", err));
                }
            };

            let html = format!(
                r#"<!DOCTYPE html>
    <div>{}</div>
"#,
                rendered
            );

            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(html)
        }
        None => {
            log::warn!("Page not found: {}", path);

            HttpResponse::NotFound()
                .content_type("text/html; charset=utf-8")
                .body("<h1>404 - Page not found</h1>")
        }
    }
}