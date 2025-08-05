
use axum::{
    extract::{State, Json},
};
use crate::application_state;
use crate::services::auth::{LoginResponse, LoginRequest, handle_login};
use tracing::info;
use crate::error::ApiError;

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
        Err(e) => { Err(e) }
    }
}