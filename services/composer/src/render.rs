use crate::{contextloader, experiment, page, AppState};
use actix_web::{web, HttpRequest, HttpResponse};
use crossbeam_channel::{unbounded, Receiver, Sender};
use deno_core::{error::AnyError, serde_v8, v8, JsRuntime, RuntimeOptions};
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
        rfa_replacements_json: String,
        response: oneshot::Sender<Result<String, AnyError>>,
    },
}

pub enum AdminCommand {
    RegisterRfa(page::RFAConfig),
    ResetRfas,
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

    pub async fn render(
        &self,
        rfa_id: &str,
        context: &Value,
        rfa_replacements: &[experiment::RfaReplacement],
    ) -> Result<String, AnyError> {
        let (tx, rx) = oneshot::channel();
        let request = RenderRequest::Render {
            rfa_id: rfa_id.to_string(),
            context_json: serde_json::to_string(context)?,
            rfa_replacements_json: serde_json::to_string(rfa_replacements)?,
            response: tx,
        };

        self.render_sender
            .send(request)
            .map_err(|_| AnyError::msg("render worker queue closed"))?;

        rx.await
            .map_err(|_| AnyError::msg("render response canceled"))?
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

    fn run(
        mut self,
        admin_cmd_receiver: Receiver<AdminCommand>,
        render_receiver: Receiver<RenderRequest>,
    ) {
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

                    AdminCommand::ResetRfas => {
                        let reset = format!("globalThis.rfaRegistry = {{}};");
                        let _ = self
                            .runtime
                            .execute_script("<rfa-reset>", deno_core::FastString::from(reset));
                    }
                }
            }

            let render_cmd = render_receiver.try_recv();
            match render_cmd {
                Ok(RenderRequest::Render {
                    rfa_id,
                    context_json,
                    rfa_replacements_json,
                    response,
                }) => {
                    self.execute_render(&rfa_id, &context_json, &rfa_replacements_json)
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

    fn execute_render(
        &mut self,
        rfa_id: &str,
        context_json: &str,
        rfa_replacements_json: &str,
    ) -> Result<String, AnyError> {
        let script = format!(
            r#"
(function() {{
    const render = globalThis.rfaRegistry[{rfa_id}];
    const rootRfaId = {rfa_id};
    const context = JSON.parse({context_json:?});
    if (context.namespace === undefined || context.namespace === null || context.namespace === "") {{
        context.namespace = rootRfaId;
    }}
    const rfaReplacements = JSON.parse({rfa_replacements_json:?});
    const namespaceMatches = function(pattern, namespace) {{
        if (pattern === undefined || pattern === null || pattern === "") {{
            return true;
        }}
        if (pattern === namespace) {{
            return true;
        }}
        if (pattern.endsWith(".*")) {{
            const prefix = pattern.slice(0, -2);
            return namespace.startsWith(prefix + ".");
        }}
        return false;
    }};
    const resolvePartialId = function(partialId, namespace) {{
        const replacement = rfaReplacements.find(function(candidate) {{
            return candidate.old === partialId && namespaceMatches(candidate.namespace, namespace);
        }});
        return replacement === undefined ? partialId : replacement.new;
    }};
    const partials = {{
        include: function(partialId, partialContext) {{
            const baseContext = partialContext === undefined ? context : partialContext;
            const baseNamespace =
                typeof baseContext.namespace === "string" && baseContext.namespace.length > 0
                    ? baseContext.namespace
                    : rootRfaId;
            const partialNamespace = baseNamespace + "." + partialId;
            const resolvedPartialId = resolvePartialId(partialId, partialNamespace);
            const partial = globalThis.rfaRegistry[resolvedPartialId];
            if (typeof partial !== "function") {{
                throw new Error("RFA not found: " + resolvedPartialId);
            }}

            const scopedContext = Object.assign({{}}, baseContext, {{ namespace: partialNamespace }});
            const output = partial(scopedContext, partials);
            return typeof output === "string" ? output : JSON.stringify(output);
        }}
    }};
    const output = render(context, partials);
    return typeof output === 'string' ? output : JSON.stringify(output);
}})()
"#,
            rfa_id = serde_json::to_string(rfa_id)?,
            context_json = context_json,
            rfa_replacements_json = rfa_replacements_json
        );

        let result = self
            .runtime
            .execute_script("<render>", deno_core::FastString::from(script))?;
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

pub async fn render_page(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let path = req.path();
    log::debug!("Render request: {}", path);

    let resolved_page_config = match experiment::resolve_page_config(&state, &req) {
        Some(resolved) => resolved,
        None => {
            log::warn!("Page not found: {}", path);
            return HttpResponse::NotFound()
                .content_type("text/html; charset=utf-8")
                .body("<h1>404 - Page not found</h1>");
        }
    };
    let page_config = resolved_page_config.page_config;

    if let Err(message) = page::validate_page_config(&page_config) {
        log::error!("Invalid page config for {}: {}", page_config.path, message);
        return HttpResponse::InternalServerError()
            .content_type("text/plain; charset=utf-8")
            .body(format!("Invalid page config: {}", message));
    }

    if page::resolve_rfa(&state, &page_config.rfa).is_none() {
        log::error!("RFA not found: {}", page_config.rfa);
        return HttpResponse::InternalServerError()
            .content_type("text/plain; charset=utf-8")
            .body(format!("RFA not found: {}", page_config.rfa));
    }

    let context = contextloader::build_context(&page_config.data);
    let rendered = match state
        .render_pool
        .render(
            &page_config.rfa,
            &context,
            &resolved_page_config.rfa_replacements,
        )
        .await
    {
        Ok(output) => output,
        Err(err) => {
            log::error!("RFA execution failed: {}", err);
            return HttpResponse::InternalServerError()
                .content_type("text/plain; charset=utf-8")
                .body(format!("RFA execution failed: {}", err));
        }
    };

    let body = format!("{}", rendered);

    let mut response = HttpResponse::Ok();
    response.content_type(page_config.content_type.clone());
    if let Some(cookie) = resolved_page_config.assignment_cookie {
        response.cookie(cookie);
    }
    response.body(body)
}

pub async fn reset_config(state: web::Data<AppState>) {
    state.render_pool.admin_senders.iter().for_each(|sender| {
        sender.send(AdminCommand::ResetRfas).ok();
    });
}
