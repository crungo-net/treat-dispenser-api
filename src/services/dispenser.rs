use crate::application_state::AppStateMutex;
use crate::application_state::DispenserStatus;
use crate::error::ApiError;
use crate::motor::{Direction, StepMode, StepperMotor, AsyncStepperMotor, stepper_nema14::StepperNema14};
use crate::utils::state_helpers::set_dispenser_status_async;
use crate::utils::{datetime, state_helpers::set_dispenser_status};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};

/// Dispenses treats by controlling GPIO pins for a stepper motor.
/// This function updates the dispenser state to "Dispensing" before starting the dispensing process.
/// It uses a background task to perform the dispensing steps without blocking the main thread and thus
/// does not affect API responsiveness.
/// After dispensing, it updates the state to "Operational" and records the last dispense time.
pub async fn dispense(app_state: AppStateMutex) -> Result<(), ApiError> {
    let motor: Arc<Box<dyn AsyncStepperMotor + Send + Sync>>;

    // query status before starting the process, done atomically to avoid race conditions
    {
        let mut state_guard = app_state.lock().await;
        match state_guard.status {
            DispenserStatus::Operational => {
                state_guard.status = DispenserStatus::Dispensing;
                motor = Arc::clone(&state_guard.motor);
            }
            DispenserStatus::Dispensing => {
                return Err(ApiError::Busy(
                    "Dispenser is already dispensing".to_string(),
                ));
            }
            DispenserStatus::Cooldown => {
                return Err(ApiError::Busy("Waiting for cooldown".to_string()));
            }
            DispenserStatus::Empty => {
                return Err(ApiError::Hardware("Dispenser is empty".to_string()));
            }
            _ => {
                return Err(ApiError::Hardware(format!(
                    "Dispenser is not operational (current status: {:?})",
                    state_guard.status
                )));
            }
        }
    }; // Lock is released here, we want to avoid holding the lock for long periods so other tasks can access the state

    info!("Dispensing treatos...");
    let app_state_clone = Arc::clone(&app_state);

    tokio::spawn(async move {
        let step_mode = StepMode::Full;
        let dir = Direction::CounterClockwise;
        let async_motor_run_result = motor
            .run_motor_degrees_async(2160.0, &dir, &step_mode, &app_state_clone)
            .await;

        if async_motor_run_result.is_err() {
            error!("Failed to run motor: {:?}", async_motor_run_result.err());
            set_error_status(&app_state_clone).await;
        } else {
            // enforce a cooldown period after operation
            set_dispenser_status_async(&app_state_clone, DispenserStatus::Cooldown).await;
            let cooldown_ms = app_state_clone.lock().await.app_config.motor_cooldown_ms;
            tokio::time::sleep(Duration::from_millis(cooldown_ms)).await;

            let mut state_guard = app_state_clone.lock().await;
            state_guard.last_dispense_time = Some(datetime::get_formatted_current_timestamp());
            state_guard.status = DispenserStatus::Operational;
            state_guard.last_step_index = Some(async_motor_run_result.unwrap());
            info!("Treatos dispensed successfully!");
        }
    });

    info!("Dispensing process started in the background.");
    Ok(())
}

async fn set_error_status(hw_state: &AppStateMutex) {
    if let Ok(mut state_guard) = hw_state.try_lock() {
        state_guard.status = DispenserStatus::Unknown;
    } else {
        // Try again after a small delay
        tokio::time::sleep(Duration::from_millis(100)).await;
        if let Ok(mut state_guard) = hw_state.try_lock() {
            state_guard.status = DispenserStatus::Unknown;
        }
    }
}
