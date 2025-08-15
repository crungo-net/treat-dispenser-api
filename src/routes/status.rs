use crate::services::status;
use crate::application_state::ApplicationState;
use axum::extract::State;
use axum::{Json, response::IntoResponse};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn detailed_health(
    State(hw_state): State<Arc<Mutex<ApplicationState>>>,
) -> impl IntoResponse {
    let status_response = status::get_status(&hw_state).await;
    Json(status_response)
}
