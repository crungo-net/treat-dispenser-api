use rppal::gpio::Gpio;
use serde::Serialize;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

pub type HwStateMutex = Arc<Mutex<DispenserState>>;

#[derive(Serialize, Debug, Clone, PartialEq)]
pub enum DispenserStatus {
    Dispensing,
    Operational,
    Jammed,
    Empty,
    Unknown,
    MotorControlError,
    NoGpio,
    Cooldown,
}

#[derive(Serialize, Debug)]
pub struct HealthStatus {
    pub gpio_available: bool,
    pub motor_operational: bool,
    pub treats_available: bool,
    pub last_dispensed: Option<String>,
    pub uptime_seconds: u64,
    pub dispenser_status: DispenserStatus,
    pub last_error_msg: Option<String>,
    pub last_error_time: Option<String>,
}

pub struct DispenserState {
    pub gpio: Option<Gpio>,
    pub status: DispenserStatus,
    pub startup_time: SystemTime,
    pub last_dispense_time: Option<String>,
    pub last_error_msg: Option<String>,
    pub last_error_time: Option<String>,
    pub last_step_index: Option<u32>,
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
            last_error_msg: None,
            last_error_time: None,
            last_step_index: None,
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
        last_error_msg: state_guard.last_error_msg.clone(),
        last_error_time: state_guard.last_error_time.clone(),
        dispenser_status: state_guard.status.clone(),
    }
}

/// Acquires a lock on the DispenserState and sets the dispenser status synchronously.
pub fn set_dispenser_status(state: &Arc<Mutex<DispenserState>>, status: DispenserStatus) {
    let mut state_guard = state.blocking_lock();
    debug!("Lock acquired on DispenserState");

    state_guard.status = status.clone();
    info!("Dispenser status set to {:?}", status);
    // lock is released here automatically when state_guard goes out of scope
}

pub async fn set_dispenser_status_async(
    state: &Arc<Mutex<DispenserState>>,
    status: DispenserStatus,
) {
    let mut state_guard = state.lock().await;
    debug!("Lock acquired on DispenserState");

    state_guard.status = status.clone();
    info!("Dispenser status set to {:?}", status);
    // lock is released here automatically when state_guard goes out of scope
}
