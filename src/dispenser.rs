use crate::error::ApiError;
use crate::state::{DispenserState};
use chrono::{DateTime, Local};
use rppal::gpio::{Level::High, Level::Low};
use std::sync::{Arc};
use std::{time::Duration};
use tracing::{debug, info, error};
use crate::state::DispenserStatus;
use tokio::sync::Mutex;
use rppal::gpio::{Gpio};



/// Dispenses treats by controlling GPIO pins for a stepper motor.
/// This function updates the dispenser state to "Dispensing" before starting the dispensing process.
/// It uses a background task to perform the dispensing steps without blocking the main thread and thus
/// does not affect API responsiveness.
/// After dispensing, it updates the state to "Operational" and records the last dispense time.
pub async fn dispense(hw_state: Arc<Mutex<DispenserState>>) -> Result<(), ApiError> {
    // Check GPIO and update status first
    {
        let mut state_guard = hw_state.lock().await;
        if state_guard.status == DispenserStatus::Dispensing {
            return Err(ApiError::Busy("Must wait for previous operation to complete".to_string()));
        }
        if state_guard.gpio.is_none() {
            return Err(ApiError::Hardware("GPIO not initialized".to_string()));
        }
        
        state_guard.status = DispenserStatus::Dispensing;
    }; // Lock is released here

    info!("Dispensing treatos...");
    
    let step_sequence = [
        [1, 0, 0, 0],
        [1, 1, 0, 0],
        [0, 1, 0, 0],
        [0, 1, 1, 0],
        [0, 0, 1, 0],
        [0, 0, 1, 1],
        [0, 0, 0, 1],
        [1, 0, 0, 1],
    ];

    // Spawn a task that manages the blocking work
    tokio::spawn(async move {
        // Move the CPU-intensive work and hardware operation to a dedicated OS thread
        // std::thread::sleep in a Tokio task blocks the entire Tokio runtime,
        // so we use spawn_blocking to run this in a separate thread.
        debug!("Starting treat dispensing logic in a blocking task...");
        let result = tokio::task::spawn_blocking(move || {
            // This runs in a separate OS thread and won't block the Tokio runtime
            match Gpio::new() {
                Ok(gpio) => {
                    let mut pin1 = match gpio.get(26).map(|p| p.into_output()) {
                        Ok(p) => p,
                        Err(e) => return Err(format!("Failed to get pin 26: {}", e)),
                    };
                    
                    let mut pin2 = match gpio.get(19).map(|p| p.into_output()) {
                        Ok(p) => p,
                        Err(e) => return Err(format!("Failed to get pin 19: {}", e)),
                    };
                    
                    let mut pin3 = match gpio.get(13).map(|p| p.into_output()) {
                        Ok(p) => p,
                        Err(e) => return Err(format!("Failed to get pin 13: {}", e)),
                    };
                    
                    let mut pin4 = match gpio.get(6).map(|p| p.into_output()) {
                        Ok(p) => p,
                        Err(e) => return Err(format!("Failed to get pin 6: {}", e)),
                    };
                    
                    // std::thread::sleep is fine here since we're in a dedicated thread
                    for _ in 0..512 {
                        for step in step_sequence.iter() {
                            pin1.write(if step[0] == 1 { High } else { Low });
                            pin2.write(if step[1] == 1 { High } else { Low });
                            pin3.write(if step[2] == 1 { High } else { Low });
                            pin4.write(if step[3] == 1 { High } else { Low });
                            std::thread::sleep(Duration::from_millis(1));
                        }
                    }
                    
                    // regardless of how long dispensing takes, we enforce a 5 second cooldown
                    std::thread::sleep(Duration::from_millis(5000));
                    
                    pin1.write(Low);
                    pin2.write(Low);
                    pin3.write(Low);
                    pin4.write(Low);
                    
                    Ok(())
                },
                Err(e) => Err(format!("Failed to create GPIO: {}", e)),
            }
        }).await;

        // Handle the result back in the async context
        match result {
            Ok(Ok(_)) => {
                info!("Treatos dispensed! Updating state...");
                
                let sys_time = std::time::SystemTime::now();
                let sys_local_datetime: DateTime<Local> = sys_time.into();
                let formatted_sys_time = sys_local_datetime.format("%Y-%m-%d %H:%M:%S").to_string();
                
                if let Ok(mut state_guard) = hw_state.try_lock() {
                    state_guard.last_dispense_time = Some(formatted_sys_time);
                    state_guard.status = DispenserStatus::Operational;
                } else {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    let mut state_guard = hw_state.lock().await;
                    state_guard.last_dispense_time = Some(formatted_sys_time);
                    state_guard.status = DispenserStatus::Operational;
                }
            },
            Ok(Err(e)) => {
                error!("Motor control error: {}", e);
                set_error_status(&hw_state).await;
            },
            Err(e) => {
                error!("Task execution error: {}", e);
                set_error_status(&hw_state).await;
            }
        }
    });

    // return immediately, the dispensing process is handled in the background
    // clients will need to poll or check the status later
    info!("Dispensing process started in the background.");
    Ok(())
}

async fn set_error_status(hw_state: &Arc<Mutex<DispenserState>>) {
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