use tracing::{info};
use crate::dispenser;
use crate::auth::Auth;
use crate::error::ApiError;
use axum::{response::IntoResponse, Json};
use axum::extract::State;
use std::sync::{Arc, Mutex};
use crate::health;

pub async fn root() -> impl IntoResponse {
    "Treat dispenser is online! Binky time!"
}


pub async fn dispense_treat(_auth: Auth, State(hw_state): State<Arc<Mutex<health::DispenserState>>>) -> Result<&'static str, ApiError> {
    dispenser::dispense(hw_state)?;
    Ok("Treat dispensed!")
}

pub async fn health_check() -> impl IntoResponse {
    "OK"
}

pub async fn detailed_health(
    State(hw_state): State<Arc<Mutex<health::DispenserState>>>,
) -> impl IntoResponse {
    let health_status = health::check_hardware(&hw_state);
    Json(health_status)
}