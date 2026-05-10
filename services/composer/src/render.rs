use crate::{contextloader, page, AppState};
use actix_web::{web, HttpRequest, HttpResponse};
use crossbeam_channel::{unbounded, Receiver, Sender};
use deno_core::{error::AnyError, JsRuntime, RuntimeOptions, serde_v8, v8};
use serde_json::Value;
use tokio::sync::oneshot;

pub struct RenderPool {
    sender: Sender<RenderRequest>,
    worker_count: usize,
}

pub enum RenderRequest {
    RegisterRfa(page::RFAConfig),
    Render {
        rfa_id: String,
        context_json: String,
        response: oneshot::Sender<Result<String, AnyError>>,
    },
}

impl RenderPool {
    pub fn new(worker_count: usize) -> Self {
        let count = worker_count.max(1);
        let (sender, receiver) = unbounded();

        for _ in 0..count {
            spawn_worker(receiver.clone());
        }

        Self {
            sender,
            worker_count: count,
        }
    }

    pub async fn register_rfa(&self, rfa: &crate::RFAConfig) -> Result<(), AnyError> {
        let rfa = rfa.clone();

        for _ in 0..self.worker_count {
            self.sender
                .send(RenderRequest::RegisterRfa(rfa.clone()))
                .map_err(|_| AnyError::msg("render worker queue closed"))?;
        }
        Ok(())
    }

    pub async fn render(&self, rfa_id: &str, context: &Value) -> Result<String, AnyError> {
        let (tx, rx) = oneshot::channel();
        let request = RenderRequest::Render {
            rfa_id: rfa_id.to_string(),
            context_json: serde_json::to_string(context)?,
            response: tx,
        };

        self.sender
            .send(request)
            .map_err(|_| AnyError::msg("render worker queue closed"))?;

        rx.await.map_err(|_| AnyError::msg("render response canceled"))?
    }
}

struct Worker {
    runtime: JsRuntime,
}

impl Worker {
    fn new() -> Self {
        let mut runtime = JsRuntime::new(RuntimeOptions::default());
        runtime
            .execute_script(
                "<init>",
                deno_core::FastString::from("globalThis.rfaRegistry = {};".to_string()),
            )
            .expect("failed to initialize runtime");

        Self { runtime }
    }

    fn run(mut self, receiver: Receiver<RenderRequest>) {
        while let Ok(request) = receiver.recv() {
            match request {
                RenderRequest::RegisterRfa(rfa) => {
                    let registration = format!(
                        "globalThis.rfaRegistry[{}] = {};",
                        serde_json::to_string(&rfa.id).unwrap(),
                        rfa.source
                    );
                    let _ = self.runtime.execute_script(
                        "<rfa-register>",
                        deno_core::FastString::from(registration),
                    );
                }
                RenderRequest::Render {
                    rfa_id,
                    context_json,
                    response,
                } => {
                    let result = self.execute_render(&rfa_id, &context_json);
                    let _ = response.send(result);
                }
            }
        }
    }

    fn execute_render(&mut self, rfa_id: &str, context_json: &str) -> Result<String, AnyError> {
        let script = format!(
            r#"const render = globalThis.rfaRegistry[{rfa_id}];
if (typeof render !== 'function') throw new Error('RFA not registered: {rfa_id}');
const context = JSON.parse({context_json:?});
const output = render(context);
if (typeof output === 'string') output;
else JSON.stringify(output);
"#,
            rfa_id = serde_json::to_string(rfa_id)?,
            context_json = context_json
        );

        let result = self.runtime.execute_script("<render>", deno_core::FastString::from(script))?;
        deno_core::scope!(scope, self.runtime);
        let local = v8::Local::new(scope, result);
        let output: String = serde_v8::from_v8(scope, local)?;
        Ok(output)
    }
}

fn spawn_worker(receiver: Receiver<RenderRequest>) {
    std::thread::spawn(move || {
        Worker::new().run(receiver);
    });
}

pub async fn render_page(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> HttpResponse {
    let path = req.path();
    log::debug!("Render request: {}", path);

    let page_config = match page::resolve_page(&state, path) {
        Some(config) => config,
        None => {
            log::warn!("Page not found: {}", path);
            return HttpResponse::NotFound()
                .content_type("text/html; charset=utf-8")
                .body("<h1>404 - Page not found</h1>");
        }
    };

    if page::resolve_rfa(&state, &page_config.rfa).is_none() {
        log::error!("RFA not found: {}", page_config.rfa);
        return HttpResponse::InternalServerError()
            .content_type("text/plain; charset=utf-8")
            .body(format!("RFA not found: {}", page_config.rfa));
    }

    let context = contextloader::build_context(&page_config.data);
    let rendered = match state.render_pool.render(&page_config.rfa, &context).await {
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
