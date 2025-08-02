use crate::sensors::power_monitor::{self, PowerReading};
use crate::state::{ApplicationState};

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;
use tracing::{error};

pub async fn check_hardware(state: &Arc<Mutex<ApplicationState>>) -> HealthStatus {
    let state_clone = Arc::clone(state);
    let mut state_guard = state_clone.lock().await;
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

    let power_reading = try_to_get_power_reading(&mut state_guard).await;

    HealthStatus {
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

async fn try_to_get_power_reading(
    state_guard: &mut ApplicationState,
) -> PowerReading {
    let dummy_reading = PowerReading {
        bus_voltage_volts: -1.0,
        current_amps: -1.0,
        power_watts: -1.0,
    };

    let power_monitor_opt = &state_guard.power_monitor;

    let power_monitor_arc = match power_monitor_opt {
        Some(monitor) => monitor,
        None => {
            error!("Power monitor is not initialized or available");
            return dummy_reading;
        }
    };

    let power_monitor_lock_result = power_monitor_arc.try_lock();
    if power_monitor_lock_result.is_err() {
        error!("Failed to acquire lock on power monitor, returning dummy reading");
        return dummy_reading;
    }
    let mut power_monitor = power_monitor_lock_result.unwrap();

    match power_monitor.get_power_reading() {
        Ok(reading) => return reading,
        Err(e) => {
            error!("Failed to get power reading: {}", e);
            return dummy_reading;
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HealthStatus {
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
