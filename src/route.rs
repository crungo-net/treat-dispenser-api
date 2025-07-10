use crate::auth::Auth;
use crate::dispenser;
use crate::error::ApiError;
use crate::state;
use axum::extract::State;
use axum::{Json, response::IntoResponse};
use std::sync::{Arc};
use tokio::sync::Mutex ;

pub async fn root() -> impl IntoResponse {
    "Treat dispenser is online! Binky time!"
}

pub async fn dispense_treat(
    _auth: Auth,
    State(hw_state): State<Arc<Mutex<state::DispenserState>>>,
) -> Result<&'static str, ApiError> {
    dispenser::dispense(hw_state).await?;
    // Return a response indicating the process has started, not completed
    Ok("Dispensing started, please wait...")
}

pub async fn health_check() -> impl IntoResponse {
    "OK"
}

pub async fn detailed_health(
    State(hw_state): State<Arc<Mutex<state::DispenserState>>>,
) -> impl IntoResponse {
    let health_status = state::check_hardware(&hw_state).await;
    Json(health_status)
}
