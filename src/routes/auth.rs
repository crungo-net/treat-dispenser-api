use crate::application_state;
use crate::error::ApiError;
use crate::services::auth::{LoginRequest, LoginResponse, handle_login};
use axum::extract::{Json, State};
use tracing::info;

pub async fn login(
    State(app_state): State<application_state::AppStateMutex>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let result = handle_login(app_state, payload.clone()).await;
    match result {
        Ok(response) => {
            info!("Login successful for user: {}", &payload.username);
            Ok(Json(response))
        }
        Err(e) => Err(e),
    }
}
