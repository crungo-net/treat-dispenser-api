use std::fmt::Display;

use crate::application_state::{self, ApplicationState, DispenserStatus};
use crate::utils::datetime;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

/// Records an error message and timestamp in the dispenser state
/// This function is asynchronous and locks the state mutex to ensure thread safety.
/// It can be used in any part of the application where an error needs to be logged.
pub async fn record_error<E: Display>(hw_state: &Arc<Mutex<ApplicationState>>, error: &E) {
    let mut state_lock = hw_state.lock().await;
    state_lock.last_error_msg = Some(error.to_string());
    state_lock.last_error_time = Some(datetime::get_formatted_current_timestamp());
}

/// Acquires a lock on the DispenserState and sets the dispenser status synchronously.
pub fn set_dispenser_status(state: &Arc<Mutex<ApplicationState>>, status: application_state::DispenserStatus) {
    let mut state_guard = state.blocking_lock();
    debug!("Lock acquired on DispenserState");

    state_guard.status = status.clone();
    info!("Dispenser status set to {:?}", status);
}

/// Sets the dispenser status asynchronously, acquiring a lock on the DispenserState.
pub async fn set_dispenser_status_async(
    state: &Arc<Mutex<ApplicationState>>,
    status: DispenserStatus,
) {
    let mut state_guard = state.lock().await;
    debug!("Lock acquired on DispenserState");

    state_guard.status = status.clone();
    info!("Dispenser status set to {:?}", status);
}
