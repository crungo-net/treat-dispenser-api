use crate::services::status;
use crate::application_state;
use axum::extract::State;
use axum::{Json, response::IntoResponse};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn detailed_health(
    State(hw_state): State<Arc<Mutex<application_state::ApplicationState>>>,
) -> impl IntoResponse {
    let health_status = status::check_hardware(&hw_state).await;
    Json(health_status)
}
