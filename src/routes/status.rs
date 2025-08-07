use crate::application_state;
use crate::services::status;
use axum::extract::State;
use axum::{Json, response::IntoResponse};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn detailed_health(
    State(hw_state): State<Arc<Mutex<application_state::ApplicationState>>>,
) -> impl IntoResponse {
    let status_response = status::check_hardware(&hw_state).await;
    Json(status_response)
}
