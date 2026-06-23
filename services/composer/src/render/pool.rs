use crossbeam_channel::{unbounded, Sender};
use deno_core::error::AnyError;
use serde_json::Value;
use tokio::sync::oneshot;

use crate::{experiment, page};

use super::worker;

#[derive(Clone)]
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

#[derive(Clone)]
pub enum AdminCommand {
    RegisterRfa(page::RFAConfig),
    ResetRfas,
}

impl RenderPool {
    pub fn new(worker_count: usize) -> Self {
        let worker_count = worker_count.max(1);
        let (render_sender, render_receiver) = unbounded();
        let admin_senders = worker::spawn_workers(worker_count, render_receiver);

        Self {
            render_sender,
            admin_senders,
        }
    }

    pub async fn register_rfa(&self, rfa: &page::RFAConfig) -> Result<(), AnyError> {
        self.broadcast_admin_command(AdminCommand::RegisterRfa(rfa.clone()))
    }

    pub async fn render(
        &self,
        rfa_id: &str,
        context: &Value,
        rfa_replacements: &[experiment::RfaReplacement],
    ) -> Result<String, AnyError> {
        let (response_sender, response_receiver) = oneshot::channel();
        self.send_render_request(rfa_id, context, rfa_replacements, response_sender)?;
        receive_render_response(response_receiver).await
    }

    pub fn reset_rfas(&self) {
        let _ = self.broadcast_admin_command(AdminCommand::ResetRfas);
    }

    fn broadcast_admin_command(&self, command: AdminCommand) -> Result<(), AnyError> {
        for sender in &self.admin_senders {
            sender
                .send(command.clone())
                .map_err(|_| AnyError::msg("admin command queue closed"))?;
        }

        Ok(())
    }

    fn send_render_request(
        &self,
        rfa_id: &str,
        context: &Value,
        rfa_replacements: &[experiment::RfaReplacement],
        response: oneshot::Sender<Result<String, AnyError>>,
    ) -> Result<(), AnyError> {
        self.render_sender
            .send(RenderRequest::Render {
                rfa_id: rfa_id.to_string(),
                context_json: serde_json::to_string(context)?,
                rfa_replacements_json: serde_json::to_string(rfa_replacements)?,
                response,
            })
            .map_err(|_| AnyError::msg("render worker queue closed"))
    }
}

async fn receive_render_response(
    response_receiver: oneshot::Receiver<Result<String, AnyError>>,
) -> Result<String, AnyError> {
    response_receiver
        .await
        .map_err(|_| AnyError::msg("render response canceled"))?
}
