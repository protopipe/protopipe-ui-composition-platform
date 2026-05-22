use crate::AppState;

use super::rfa_config::RFAConfig;

pub fn resolve_rfa(state: &AppState, rfa_id: &str) -> Option<RFAConfig> {
    let rfas = state.rfas.lock().unwrap();
    rfas.get(rfa_id).cloned()
}
