use crate::auth::Auth;
use crate::dispenser;
use crate::error::ApiError;
use crate::state;
use axum::extract::State;
use axum::{Json, response::IntoResponse};
use std::sync::{Arc};
use tokio::sync::Mutex ;
use chrono::{DateTime, Local};

pub async fn root() -> impl IntoResponse {
    "Treat dispenser is online! Binky time!"
}

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
            return Err(e)
        },
    };
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
