use rppal::gpio::Gpio;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tracing::{info, error};

#[derive(Serialize, Debug, Clone)]
pub enum DispenserStatus {
    Operational,
    Jammed,
    Empty,
    Disconnected,
    Unknown,
}

#[derive(Serialize, Debug)]
pub struct HealthStatus {
    pub hardware_connected: bool,
    pub motor_operational: bool,
    pub treats_available: bool,
    pub last_dispensed: Option<String>,
    pub uptime_seconds: u64,
}

pub struct DispenserState {
    pub gpio: Option<Gpio>,
    pub status: DispenserStatus,
    pub startup_time: SystemTime,
    pub last_dispense_time: Option<String>,
}

impl DispenserState {
    pub fn new() -> Self {
        let gpio = match Gpio::new() {
            Ok(gpio) => {
                info!("GPIO initialized successfully");
                Some(gpio)
            }
            Err(e) => {
                error!("Failed to initialize GPIO: {}", e);
                None    
            }
        };

        Self {
            gpio,
            status: DispenserStatus::Unknown,
            startup_time: SystemTime::now(),
            last_dispense_time: None,
        }
    }
}

pub fn check_hardware(state: &Arc<Mutex<DispenserState>>) -> HealthStatus {
    let state_guard = state.lock().unwrap();
    let now = SystemTime::now();

    let hw_connected = state_guard.gpio.is_some();

    let treats_available = match &state_guard.gpio {
        Some(gpio) => {
            // Placeholder for actual GPIO pin check logic
            // In a real application, this would check if the treat dispenser has treats available
            true
        }
        None => false,
    };

    let motor_operational = match &state_guard.gpio {
        Some(gpio) => {
            // Placeholder for actual motor operational check logic
            // In a real application, this would check if the motor is functioning correctly
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
        hardware_connected: hw_connected,
        motor_operational: motor_operational,
        treats_available: treats_available,
        last_dispensed: last_dispensed,
        uptime_seconds: uptime_seconds,
    }
}

