use tracing::{info, debug};
use crate::error::ApiError;
use std::sync::{Arc, Mutex};
use crate::health::DispenserState;

pub fn dispense(hw_state: Arc<Mutex<DispenserState>>) -> Result<(), ApiError> {
    // try to acquire lock on the hardware state, propagating an error if it fails
    let state_guard = hw_state.lock().map_err(
        |_| ApiError::Hardware("Failed to lock hardware state".to_string())
    )?;

    let gpio = match &state_guard.gpio {
        Some(gpio) => gpio,
        None => return Err(ApiError::Hardware("GPIO not initialized".to_string())),
    };

    // todo: Implement hardware interaction to dispense a treat
    let hardware_success = false; // Simulate hardware interaction

    if !hardware_success {
        return Err(ApiError::Hardware("Failed to dispense treat".to_string()));
    }
    info!("Treat dispensed!");
    Ok(())
}