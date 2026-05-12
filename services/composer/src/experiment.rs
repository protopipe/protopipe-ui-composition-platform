use crate::page;
use crate::AppState;
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct ExperimentConfig {
    pub id: String,
    pub variants: Vec<Variant>,
}

#[derive(Serialize, Deserialize)]
pub struct ExperimentConfigDto {
    pub experiment_id: String,
    pub variants: Vec<VariantDto>,
}

#[derive(Serialize, Deserialize)]
pub struct VariantDto {
    pub id: String,
    pub weight: u32,
}

pub struct Variant {
    pub id: String,
    pub weight: u32,
    pub overrideConfig: page::PageConfig,
}

pub enum Override {
    StaticData {
        value: page::StaticData,
    },
    DynamicRestData {
        value: page::DynamicRestData,
    },
    Rfa {
        value: page::RFAConfig,
    },
}

pub async fn register_experiment(state: web::Data<AppState>, config: web::Json<ExperimentConfigDto>) -> HttpResponse {
    // For each variant, we need to register the overridden page config and RFA if they exist.

    HttpResponse::Ok().json(serde_json::json!({"status": "Fake it"}))
}

pub async fn get_experiments(state: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"status": "Fake it"}))
}

pub async fn reset_config(state: web::Data<AppState>) {
}