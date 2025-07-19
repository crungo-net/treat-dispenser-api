use crate::error::ApiError;
use crate::middleware::auth::Auth;
use crate::services::dispenser;
use crate::state;
use crate::utils::state_helpers;
use axum::extract::State;
use std::sync::Arc;

pub async fn dispense_treat(
    _auth: Auth,
    State(hw_state): State<state::AppStateMutex>,
) -> Result<&'static str, ApiError> {
    let hw_state_clone = Arc::clone(&hw_state);

    match dispenser::dispense(hw_state_clone).await {
        Ok(_) => (),
        Err(e) => {
            state_helpers::record_error(&hw_state, &e).await;
            return Err(e);
        }
    };
    Ok("Dispensing started, please wait...")
}
