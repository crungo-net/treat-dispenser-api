use crate::sensors::power_monitor::{PowerReading};
use crate::application_state::{ApplicationState};

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;
use tracing::{error};

pub async fn check_hardware(state: &Arc<Mutex<ApplicationState>>) -> StatusResponse {
    let now = SystemTime::now();

    // Lock once and extract only what we need
    let (
        gpio_available,
        startup_time,
        last_dispensed,
        last_error_msg,
        last_error_time,
        dispenser_status,
        version,
        motor_name,
        mut power_readings_rx,
    ) = {
        let state_guard = state.lock().await;

        (
            state_guard.gpio.is_some(),
            state_guard.startup_time,
            state_guard.last_dispense_time.clone(),
            state_guard.last_error_msg.clone(),
            state_guard.last_error_time.clone(),
            state_guard.status.clone().to_string(),
            state_guard.version.clone(),
            state_guard.motor.get_name().clone(),
            state_guard.power_readings_tx.subscribe(),
        )
    }; // lock is dropped here

    let uptime_seconds = now
        .duration_since(startup_time)
        .unwrap_or_default()
        .as_secs();

    // wait for up to 1500ms for a power reading from broadcast channel
    let power_reading = match tokio::time::timeout(Duration::from_millis(1500), power_readings_rx.recv()).await {
        Ok(Ok(reading)) => reading,
        Ok(Err(e)) => {
            error!("Broadcast receive error: {}", e);
            PowerReading::dummy()
        }
        Err(_) => {
            error!("Timed out waiting for power reading");
            PowerReading::dummy()
        }
    };

    StatusResponse {
        gpio_available,
        motor_operational: gpio_available, // temporary placeholder
        treats_available: gpio_available,  // temporary placeholder
        last_dispensed,
        uptime_seconds,
        last_error_msg,
        last_error_time,
        dispenser_status,
        version,
        motor: motor_name,
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
