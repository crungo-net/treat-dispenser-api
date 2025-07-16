use crate::state;
use axum::extract::State;
use axum::{Json, response::IntoResponse};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn detailed_health(
    State(hw_state): State<Arc<Mutex<state::DispenserState>>>,
) -> impl IntoResponse {
    let health_status = state::check_hardware(&hw_state).await;
    Json(health_status)
}
