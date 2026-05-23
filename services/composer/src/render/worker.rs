use crossbeam_channel::{unbounded, Receiver, Sender};
use deno_core::{error::AnyError, serde_v8, v8, JsRuntime, RuntimeOptions};

use super::pool::{AdminCommand, RenderRequest};
use super::scripts;

pub fn spawn_workers(
    worker_count: usize,
    render_receiver: Receiver<RenderRequest>,
) -> Vec<Sender<AdminCommand>> {
    (0..worker_count)
        .map(|_| spawn_worker(render_receiver.clone()))
        .collect()
}

struct Worker {
    runtime: JsRuntime,
}

impl Worker {
    fn new() -> Self {
        let mut runtime = JsRuntime::new(RuntimeOptions::default());
        runtime
            .execute_script("<init>", scripts::initialize_runtime())
            .expect("failed to initialize runtime");

        Self { runtime }
    }

    fn run(
        mut self,
        admin_receiver: Receiver<AdminCommand>,
        render_receiver: Receiver<RenderRequest>,
    ) {
        loop {
            self.drain_admin_commands(&admin_receiver);
            self.receive_render_request(&render_receiver);
        }
    }

    fn drain_admin_commands(&mut self, admin_receiver: &Receiver<AdminCommand>) {
        while let Ok(command) = admin_receiver.try_recv() {
            self.handle_admin_command(command);
        }
    }

    fn handle_admin_command(&mut self, command: AdminCommand) {
        match command {
            AdminCommand::RegisterRfa(rfa) => self.register_rfa(&rfa),
            AdminCommand::ResetRfas => self.reset_rfas(),
        }
    }

    fn register_rfa(&mut self, rfa: &crate::RFAConfig) {
        let _ = self
            .runtime
            .execute_script("<rfa-register>", scripts::register_rfa(rfa));
    }

    fn reset_rfas(&mut self) {
        let _ = self
            .runtime
            .execute_script("<rfa-reset>", scripts::reset_registry());
    }

    fn receive_render_request(&mut self, render_receiver: &Receiver<RenderRequest>) {
        match render_receiver.try_recv() {
            Ok(request) => self.handle_render_request(request),
            Err(_) => sleep_until_next_poll(),
        }
    }

    fn handle_render_request(&mut self, request: RenderRequest) {
        match request {
            RenderRequest::Render {
                rfa_id,
                context_json,
                rfa_replacements_json,
                response,
            } => {
                let render_result =
                    self.execute_render(&rfa_id, &context_json, &rfa_replacements_json);
                send_render_response(response, render_result);
            }
        }
    }

    fn execute_render(
        &mut self,
        rfa_id: &str,
        context_json: &str,
        rfa_replacements_json: &str,
    ) -> Result<String, AnyError> {
        let result = self.runtime.execute_script(
            "<render>",
            scripts::render_rfa(rfa_id, context_json, rfa_replacements_json)?,
        )?;
        deno_core::scope!(scope, self.runtime);
        let local = v8::Local::new(scope, result);
        let output: String = serde_v8::from_v8(scope, local)?;
        Ok(output)
    }
}

fn spawn_worker(render_receiver: Receiver<RenderRequest>) -> Sender<AdminCommand> {
    let (admin_sender, admin_receiver) = unbounded();

    std::thread::spawn(move || {
        Worker::new().run(admin_receiver, render_receiver);
    });

    admin_sender
}

fn send_render_response(
    response: tokio::sync::oneshot::Sender<Result<String, AnyError>>,
    render_result: Result<String, AnyError>,
) {
    if let Err(err) = &render_result {
        log::error!("Render error: {}", err);
    }

    let _ = response.send(render_result);
}

fn sleep_until_next_poll() {
    // TODO expose metrics about idle time and sleep for a bit to avoid busy waiting
    std::thread::sleep(std::time::Duration::from_millis(10));
}
