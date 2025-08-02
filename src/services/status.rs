use crate::sensors::power_monitor::{PowerReading};
use crate::application_state::{ApplicationState};

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;
use tracing::{error};

pub async fn check_hardware(state: &Arc<Mutex<ApplicationState>>) -> StatusResponse {
    let state_guard = state.lock().await;
    let mut power_readings_rx = state_guard.power_readings_tx.subscribe();

    let now = SystemTime::now();

    let gpio_available = state_guard.gpio.is_some();

    let treats_available = match &state_guard.gpio {
        Some(_gpio) => {
            // Placeholder for sensor logic to check if treats are available
            true
        }
        None => false,
    };

    let motor_operational = match &state_guard.gpio {
        Some(_gpio) => {
            // Placeholder for actual motor operational check logic
            true
        }
        None => false,
    };

    let uptime_seconds = now
        .duration_since(state_guard.startup_time)
        .unwrap_or_default()
        .as_secs();

    let last_dispensed = state_guard.last_dispense_time.clone();

    // wait for up to 1500ms for a power reading from broadcast channel
    let power_reading_result = match tokio::time::timeout(Duration::from_millis(1500), power_readings_rx.recv()).await {
        Ok(reading) => reading,
        Err(e) => {
            error!("Failed to receive power reading: {}", e);
            Ok(
                PowerReading {
                bus_voltage_volts: -1.0,
                current_amps: -1.0,
                power_watts: -1.0,
            })
        }
    };
    let power_reading = power_reading_result.unwrap();

    StatusResponse {
        gpio_available,
        motor_operational: motor_operational,
        treats_available: treats_available,
        last_dispensed: last_dispensed,
        uptime_seconds: uptime_seconds,
        last_error_msg: state_guard.last_error_msg.clone(),
        last_error_time: state_guard.last_error_time.clone(),
        dispenser_status: state_guard.status.clone().to_string(),
        version: state_guard.version.clone(),
        motor: state_guard.motor.get_name().clone(),
        motor_voltage_volts: Some(power_reading.bus_voltage_volts),
        motor_current_amps: Some(power_reading.current_amps),
        motor_power_watts: Some(power_reading.power_watts),
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StatusResponse {
    pub gpio_available: bool,
    pub motor_operational: bool,
    pub treats_available: bool,
    pub last_dispensed: Option<String>,
    pub uptime_seconds: u64,
    pub dispenser_status: String,
    pub last_error_msg: Option<String>,
    pub last_error_time: Option<String>,
    pub version: String,
    pub motor: String,
    pub motor_voltage_volts: Option<f32>,
    pub motor_current_amps: Option<f32>,
    pub motor_power_watts: Option<f32>,
}
