use std::sync::Arc;
use tokio::sync::Mutex;
use crate::state::{ApplicationState, HealthStatus};
use std::time::SystemTime;

pub async fn check_hardware(state: &Arc<Mutex<ApplicationState>>) -> HealthStatus {
    let state_guard = state.lock().await;
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

    HealthStatus {
        gpio_available,
        motor_operational: motor_operational,
        treats_available: treats_available,
        last_dispensed: last_dispensed,
        uptime_seconds: uptime_seconds,
        last_error_msg: state_guard.last_error_msg.clone(),
        last_error_time: state_guard.last_error_time.clone(),
        dispenser_status: state_guard.status.clone().to_string(),
    }
}