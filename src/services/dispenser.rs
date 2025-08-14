use crate::application_state::AppStateMutex;
use crate::application_state::DispenserStatus;
use crate::error::ApiError;
use crate::motor::{AsyncStepperMotor, Direction, StepMode};
use crate::utils::datetime;
use crate::utils::state_helpers::set_dispenser_status_async;
use crate::config;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};

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
            DispenserStatus::Operational | DispenserStatus::Cancelled => {
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
        let cancel_token = {
            let token = CancellationToken::new();
            // short lock to set the cancellation token
            app_state_clone.lock().await.motor_cancel_token = Some(token.clone());
            token
        };

        let step_mode = StepMode::Full;
        let dir = Direction::CounterClockwise;
        let async_motor_run_result = motor
            .run_motor_degrees_async(2160.0, &dir, &step_mode, &app_state_clone, &cancel_token)
            .await;

        match async_motor_run_result {
            Ok(steps) => {
                info!("Motor run completed successfully, steps: {}", steps);
                // enforce a cooldown period after operation
                set_dispenser_status_async(&app_state_clone, DispenserStatus::Cooldown).await;
                let cooldown_ms = app_state_clone.lock().await.app_config.motor_cooldown_ms.unwrap_or(config::MOTOR_COOLDOWN_MS_DEFAULT);
                tokio::time::sleep(Duration::from_millis(cooldown_ms)).await;

                let mut state_guard = app_state_clone.lock().await;
                state_guard.last_dispense_time = Some(datetime::get_formatted_current_timestamp());
                state_guard.status = DispenserStatus::Operational;
                state_guard.last_step_index = Some(async_motor_run_result.unwrap());
                info!("Treatos dispensed successfully!");
            }
            Err(e) => {
                warn!("Motor operation ended: {:?}", e);
                if cancel_token.is_cancelled() {
                    warn!("Motor operation was cancelled.");
                    set_dispenser_status_async(&app_state_clone, DispenserStatus::Cancelled).await;
                } else {
                    set_dispenser_status_async(&app_state_clone, DispenserStatus::Unknown).await;
                }
            }
        }

        // Clear the cancellation token after dispensing
        {
            let mut state_guard = app_state_clone.lock().await;
            state_guard.motor_cancel_token = None;
            debug!("Motor cancellation token cleared after dispensing.");
        }
    });

    info!("Dispensing process started in the background.");
    Ok(())
}

pub async fn cancel_dispense(app_state: AppStateMutex) -> Result<(), ApiError> {
    let mut state_guard = app_state.lock().await;

    if let Some(cancel_token) = &state_guard.motor_cancel_token {
        cancel_token.cancel();
        info!("Motor operation cancelled successfully.");
        state_guard.status = DispenserStatus::Cancelled;
        state_guard.motor_cancel_token = None;
    } else {
        return Err(ApiError::Hardware(
            "No ongoing motor operation to cancel".to_string(),
        ));
    }

    Ok(())
}
