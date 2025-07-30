use rppal::gpio::Gpio;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::AppConfig;
use crate::motor::StepperMotor;
use crate::motor::stepper_28byj48::Stepper28BYJ48;
use crate::motor::stepper_mock::StepperMock;
use crate::motor::stepper_nema14::StepperNema14;

pub type AppStateMutex = Arc<Mutex<ApplicationState>>;

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

impl fmt::Display for DispenserStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
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
}
pub struct ApplicationState {
    pub gpio: Option<Gpio>,
    pub status: DispenserStatus,
    pub startup_time: SystemTime,
    pub last_dispense_time: Option<String>,
    pub last_error_msg: Option<String>,
    pub last_error_time: Option<String>,
    pub last_step_index: Option<u32>,
    pub motor: Arc<Box<dyn StepperMotor + Send + Sync>>,
    pub app_config: AppConfig,
}

impl ApplicationState {
    pub fn new(app_config: AppConfig) -> Self {
        let status: DispenserStatus;

        // Initialize motor here, assuming a default implementation exists
        let motor_env =
            std::env::var("MOTOR_TYPE").unwrap_or_else(|_| "Stepper28BYJ48".to_string());

        let motor = match init_motor(motor_env.to_string(), app_config.clone()) {
            Ok(motor) => {
                info!("Motor initialized: {}", motor.get_name());
                Arc::new(motor)
            }
            Err(e) => {
                error!("Failed to select motor: {}", e);
                std::process::exit(1);
            }
        };

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

        if motor.requires_gpio() && gpio.is_none() {
            error!("Motor requires GPIO but GPIO initialization failed");
            status = DispenserStatus::NoGpio;
        } else {
            status = DispenserStatus::Operational;
        }

        Self {
            gpio,
            status,
            startup_time: SystemTime::now(),
            last_dispense_time: None,
            last_error_msg: None,
            last_error_time: None,
            last_step_index: None,
            motor,
            app_config,
        }
    }
}

/// Acquires a lock on the DispenserState and sets the dispenser status synchronously.
pub fn set_dispenser_status(state: &Arc<Mutex<ApplicationState>>, status: DispenserStatus) {
    let mut state_guard = state.blocking_lock();
    state_guard.status = status.clone();
    info!("Dispenser status set to {:?}", status);
    // lock is released here automatically when state_guard goes out of scope
}

pub async fn set_dispenser_status_async(
    state: &Arc<Mutex<ApplicationState>>,
    status: DispenserStatus,
) {
    let mut state_guard = state.lock().await;
    state_guard.status = status.clone();
    info!("Dispenser status set to {:?}", status);
    // lock is released here automatically when state_guard goes out of scope
}

fn init_motor(
    motor_type: String,
    config: AppConfig,
) -> Result<Box<dyn StepperMotor + Send + Sync>, String> {
    match motor_type.as_str() {
        "Stepper28BYJ48" => Ok(Box::new(Stepper28BYJ48::new())),
        "StepperNema14" => {
            if config.nema14.is_none() {
                return Err("Nema14 configuration is missing".to_string());
            }
            let nema14_config = config.nema14.clone().unwrap();
            Ok(Box::new(StepperNema14::new(nema14_config)))
        }
        "StepperMock" => Ok(Box::new(StepperMock::new())),
        // Add more motor types here as needed
        _ => Err(format!("Unsupported motor type '{}'", motor_type)),
    }
}
