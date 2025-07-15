use axum::extract::State;
use std::sync::Arc;
use chrono::{DateTime, Local};
use crate::middleware::auth::Auth;
use crate::state;
use crate::error::ApiError;
use crate::services::dispenser;


pub async fn dispense_treat(
    _auth: Auth,
    State(hw_state): State<state::HwStateMutex>,
) -> Result<&'static str, ApiError> {
    let hw_state_clone = Arc::clone(&hw_state);

    match dispenser::dispense(hw_state_clone).await {
        Ok(_) => (),
        Err(e) => {
            let mut state_lock = hw_state.lock().await;
            let sys_time = std::time::SystemTime::now();
            let sys_local_datetime: DateTime<Local> = sys_time.into();
            let formatted_sys_time = sys_local_datetime.format("%Y-%m-%d %H:%M:%S").to_string();

            state_lock.last_error_msg = Some(e.to_string());
            state_lock.last_error_time = Some(formatted_sys_time);
            return Err(e);
        }
    };
    Ok("Dispensing started, please wait...")
}