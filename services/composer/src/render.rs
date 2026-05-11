use crate::{contextloader, page, AppState};
use actix_web::{web, HttpRequest, HttpResponse};
use crossbeam_channel::{unbounded, Receiver, Sender};
use deno_core::{error::AnyError, JsRuntime, RuntimeOptions, serde_v8, v8};
use serde_json::Value;
use tokio::sync::oneshot;

pub struct RenderPool {
    render_sender: Sender<RenderRequest>,
    admin_senders: Vec<Sender<AdminCommand>>,
}

pub enum RenderRequest {
    Render {
        rfa_id: String,
        context_json: String,
        response: oneshot::Sender<Result<String, AnyError>>,
    },
}

pub enum AdminCommand {
    RegisterRfa(page::RFAConfig),
}

impl RenderPool {
    pub fn new(worker_count: usize) -> Self {
        let count = worker_count.max(1);
        let (render_sender, render_receiver) = unbounded();
        let mut admin_senders = Vec::with_capacity(count);

        for _ in 0..count {
            let (admin_sender, admin_receiver) = unbounded();
            admin_senders.push(admin_sender);
            spawn_worker(admin_receiver, render_receiver.clone());
        }

        Self {
            render_sender,
            admin_senders: admin_senders,
        }
    }

    pub async fn register_rfa(&self, rfa: &crate::RFAConfig) -> Result<(), AnyError> {
        let rfa = rfa.clone();

        for sender in &self.admin_senders {
            sender
                .send(AdminCommand::RegisterRfa(rfa.clone()))
                .map_err(|_| AnyError::msg("admin command queue closed"))?;
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

        self.render_sender
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

    fn run(mut self, admin_cmd_receiver: Receiver<AdminCommand>, render_receiver: Receiver<RenderRequest>) {

        loop {
            while let Ok(cmd) = admin_cmd_receiver.try_recv() {
                match cmd {
                    AdminCommand::RegisterRfa(rfa) => {
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
                }
            }

            let render_cmd = render_receiver.try_recv();
            match render_cmd {
                Ok(RenderRequest::Render { rfa_id, context_json, response }) => {
                    self.execute_render(&rfa_id, &context_json)
                        .map_err(|e| log::error!("Render error: {}", e))
                        .ok()
                        .and_then(|output| response.send(Ok(output)).ok());
                }
                Err(_) => {
                    // TODO expose metrics about idle time and sleep for a bit to avoid busy waiting
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }
    }

    fn execute_render(&mut self, rfa_id: &str, context_json: &str) -> Result<String, AnyError> {
        let script = format!(
            r#"
(function() {{
    const render = globalThis.rfaRegistry[{rfa_id}];
    const context = JSON.parse({context_json:?});
    const output = render(context);
    return typeof output === 'string' ? output : JSON.stringify(output);
}})()
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

fn spawn_worker(admin_rx: Receiver<AdminCommand>, render_rx: Receiver<RenderRequest>) {
    std::thread::spawn(move || {
        Worker::new().run(admin_rx, render_rx);
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
