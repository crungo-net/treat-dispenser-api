use crate::error::ApiError;
use crate::motor::{Direction, StepMode, StepperMotor};
use crate::state::DispenserStatus;
use crate::state::{AppStateMutex, set_dispenser_status};
use crate::utils::datetime;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};

/// Dispenses treats by controlling GPIO pins for a stepper motor.
/// This function updates the dispenser state to "Dispensing" before starting the dispensing process.
/// It uses a background task to perform the dispensing steps without blocking the main thread and thus
/// does not affect API responsiveness.
/// After dispensing, it updates the state to "Operational" and records the last dispense time.
pub async fn dispense(app_state: AppStateMutex) -> Result<(), ApiError> {
    let motor: Arc<Box<dyn StepperMotor + Send + Sync>>;

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

    // Spawn a task that manages the blocking work
    tokio::spawn(async move {
        // Handle the blocking motor control in a separate thread
        let motor_task_result = tokio::task::spawn_blocking(move || {
            let step_mode = StepMode::Full;
            let dir = Direction::CounterClockwise;
            let result = motor.run_motor_degrees(360.0, &dir, &step_mode);

            // enforce a cooldown period after operation
            set_dispenser_status(&app_state_clone, DispenserStatus::Cooldown);
            let cooldown_ms = app_state_clone.blocking_lock().app_config.motor_cooldown_ms;
            std::thread::sleep(Duration::from_millis(cooldown_ms));

            result
        })
        .await;

        // Handle the result back in the async context
        match motor_task_result {
            Ok(Ok(last_step_index)) => {
                info!("Treatos dispensed successfully!");
                debug!("Last step index reached: {}", last_step_index);

                let mut state_guard = app_state.lock().await;
                state_guard.last_dispense_time = Some(datetime::get_formatted_current_timestamp());
                state_guard.status = DispenserStatus::Operational;
                state_guard.last_step_index = Some(last_step_index);
                debug!(
                    "Dispenser state updated: last_dispense_time={:?}, status={:?}, last_step_index={:?}",
                    state_guard.last_dispense_time, state_guard.status, state_guard.last_step_index
                );
            }
            Ok(Err(e)) => {
                error!("Motor control error: {}", e);
                set_error_status(&app_state).await;
            }
            Err(e) => {
                error!("Task execution error: {}", e);
                set_error_status(&app_state).await;
            }
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
