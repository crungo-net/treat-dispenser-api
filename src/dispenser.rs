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

    // Spawn a task that manages the blocking work
    tokio::spawn(async move {
        // Handle the blocking motor control in a separate thread
        let result = tokio::task::spawn_blocking(move || {
            trigger_motor(512)
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

fn trigger_motor(number_of_steps: u16) -> Result<(), String> {
    // this sequence is for a 4-phase stepper motor, where each sub-array represents
    // the state of the pins [pin1, pin2, pin3, pin4]
    // 1 means HIGH, 0 means LOW
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

    match Gpio::new() {
        Ok(gpio) => {
            let mut pin1 = get_pin(&gpio, 26)?;
            let mut pin2 = get_pin(&gpio, 19)?;
            let mut pin3 = get_pin(&gpio, 13)?;
            let mut pin4 = get_pin(&gpio, 6)?;  
            
            info!("Starting motor with {} steps", number_of_steps);
            for _ in 0..number_of_steps {
                for step in step_sequence.iter() {
                    pin1.write(step[0].into());
                    pin2.write(step[1].into());
                    pin3.write(step[2].into());
                    pin4.write(step[3].into());
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
            info!("Motor operation completed, entering cooldown period");
            
            // regardless of how long dispensing takes, we enforce a 5 second cooldown
            std::thread::sleep(Duration::from_millis(5000));
            
            pin1.write(Low);
            pin2.write(Low);
            pin3.write(Low);
            pin4.write(Low);
            
            Ok(())
        },
        Err(e) => Err(format!("Failed to create local Gpio instance: {}", e)),
    }
}

fn get_pin(gpio: &Gpio, gpio_pin_num: u8) -> Result<rppal::gpio::OutputPin, String> {
    gpio.get(gpio_pin_num)
        .map(|p| p.into_output())
        .map_err(|e| format!("Failed to get pin {}: {}", gpio_pin_num, e))
}