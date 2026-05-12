use actix_web::{web, HttpResponse};
use crate::AppState;
use crate::page;
use crate::experiment;
use crate::render;


pub async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"status": "OK"}))
}

pub async fn reset_config(state: web::Data<AppState>) -> HttpResponse {
    // This endpoint will be called by tests to reset the state of the service before each scenario.
    // It should clear all registered pages and RFAs.

    page::reset_config(state.clone()).await; 
    experiment::reset_config(state.clone()).await;
    render::reset_config(state.clone()).await;

    HttpResponse::Ok().json(serde_json::json!({"status": "Config reset successfully"}))
}