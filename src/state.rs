use rppal::gpio::Gpio;
use serde::Serialize;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;
use tracing::{error, info};

#[derive(Serialize, Debug, Clone, PartialEq)]
pub enum DispenserStatus {
    Dispensing,
    Operational,
    Jammed,
    Empty,
    Unknown,
    MotorControlError,
    NoGpio,
}

#[derive(Serialize, Debug)]
pub struct HealthStatus {
    pub gpio_available: bool,
    pub motor_operational: bool,
    pub treats_available: bool,
    pub last_dispensed: Option<String>,
    pub uptime_seconds: u64,
    pub dispenser_status: DispenserStatus,
}

pub struct DispenserState {
    pub gpio: Option<Gpio>,
    pub status: DispenserStatus,
    pub startup_time: SystemTime,
    pub last_dispense_time: Option<String>,
}

impl DispenserState {
    pub fn new() -> Self {
        let status: DispenserStatus;

        let gpio = match Gpio::new() {
            Ok(gpio) => {
                info!("GPIO initialized successfully");
                status = DispenserStatus::Operational;
                Some(gpio)
            }
            Err(e) => {
                error!("Failed to initialize GPIO: {}", e);
                status = DispenserStatus::NoGpio;
                None
            }
        };


        Self {
            gpio,
            status,
            startup_time: SystemTime::now(),
            last_dispense_time: None,
        }
    }

}

pub async fn check_hardware(state: &Arc<Mutex<DispenserState>>) -> HealthStatus {
    let state_guard = state.lock().await;
    let now = SystemTime::now();

    let gpio_available = state_guard.gpio.is_some();

    let treats_available = match &state_guard.gpio {
        Some(gpio) => {
            // Placeholder for sensor logic to check if treats are available
            true
        }
        None => false,
    };

    let motor_operational = match &state_guard.gpio {
        Some(gpio) => {
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
        dispenser_status: state_guard.status.clone(),
    }
}
