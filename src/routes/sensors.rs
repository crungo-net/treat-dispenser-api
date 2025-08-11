use std::sync::Arc;

use crate::application_state;
use crate::error::ApiError;
use crate::services::weight_monitor::{self, TareResponse};
use crate::utils::state_helpers;
use axum::extract::State;
use axum::Json;

pub async fn tare_weight_sensor(
    State(app_state): State<application_state::AppStateMutex>,
) -> Result<Json<TareResponse>, ApiError> {
    let app_state = Arc::clone(&app_state);

    let tare_result = weight_monitor::tare_weight_sensor(Arc::clone(&app_state)).await;

     match tare_result {
        Ok(response) => {
            Ok(Json(response))
        }
        Err(e) => {
            state_helpers::record_error(&app_state, &e).await;
            Err(ApiError::Hardware(e))
        }
    }
}