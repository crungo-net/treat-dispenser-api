use rppal::gpio::Gpio;
use rppal::spi::Bus;
use rppal::spi::SlaveSelect;
use serde::Serialize;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::SystemTime;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use crate::AppConfig;
use crate::motor::AsyncStepperMotor;
use crate::motor::stepper_28byj48::Stepper28BYJ48;
use crate::motor::stepper_mock::StepperMock;
use crate::motor::stepper_nema14::StepperNema14;
use crate::sensors::PowerReading;
use crate::sensors::PowerSensor;
use crate::sensors::WeightReading;
use crate::sensors::WeightSensor;
use crate::sensors::WeightSensorCalibration;
use crate::services::weight_monitor;

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
    Cancelled,
    Calibrating,
    CalibrationFailed,
}

impl fmt::Display for DispenserStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct ApplicationState {
    pub gpio: Option<Gpio>,
    pub status: DispenserStatus,
    pub startup_time: SystemTime,
    pub last_dispense_time: Option<String>,
    pub last_error_msg: Option<String>,
    pub last_error_time: Option<String>,
    pub last_step_index: Option<u32>,
    pub motor: Arc<Box<dyn AsyncStepperMotor + Send + Sync>>,
    pub app_config: AppConfig,
    pub version: String,
    pub power_sensor_mutex: Option<Arc<Mutex<Box<dyn PowerSensor>>>>,
    pub power_readings_tx: tokio::sync::watch::Sender<PowerReading>,
    pub power_readings_rx: tokio::sync::watch::Receiver<PowerReading>,
    pub motor_cancel_token: Option<CancellationToken>,
    pub weight_sensor_mutex: Option<Arc<Mutex<Box<dyn WeightSensor>>>>,
    pub weight_readings_tx: tokio::sync::watch::Sender<WeightReading>,
    pub weight_readings_rx: tokio::sync::watch::Receiver<WeightReading>,
    pub calibration_in_progress: Arc<AtomicBool>,
    pub calibration_tx: tokio::sync::watch::Sender<WeightSensorCalibration>,
    pub calibration_rx: tokio::sync::watch::Receiver<WeightSensorCalibration>,
}

impl ApplicationState {
    pub fn new(app_config: AppConfig) -> Self {
        let version = env!("CARGO_PKG_VERSION").to_string();
        let status: DispenserStatus;

        info!("Starting treat-dispenser-api, version: {}", version);

        let motor = match init_motor(&app_config) {
            Ok(motor) => {
                info!("Motor initialized: {}", motor.get_name());
                Arc::new(motor)
            }
            Err(e) => {
                error!("Failed to select motor: {}", e);
                std::process::exit(1);
            }
        };

        let power_sensor_mutex = match init_power_sensor(&app_config) {
            Ok(sensor) => Some(Arc::new(Mutex::new(sensor))),
            Err(e) => {
                error!("Failed to initialize power sensor: {}", e);
                None
            }
        };

        let (power_readings_tx, power_readings_rx) =
            tokio::sync::watch::channel(PowerReading::default());

        let gpio = match Gpio::new() {
            Ok(gpio) => {
                info!("GPIO initialized successfully");
                Some(gpio)
            }
            Err(e) => {
                warn!("Failed to initialize GPIO: {}", e);
                None
            }
        };

        if motor.requires_gpio() && gpio.is_none() {
            error!("Motor requires GPIO but GPIO initialization failed");
            status = DispenserStatus::NoGpio;
        } else {
            status = DispenserStatus::Operational;
        }

        let weight_sensor_result = init_weight_sensor(&app_config);
        let weight_sensor = match weight_sensor_result {
            Ok(sensor) => sensor,
            Err(e) => {
                error!("Failed to initialize weight sensor: {}", e);
                std::process::exit(1)
            }
        };

        let weight_sensor_mutex = Some(Arc::new(Mutex::new(weight_sensor)));
        let (weight_readings_tx, weight_readings_rx) =
            tokio::sync::watch::channel(WeightReading::default());

        let weight_sensor_calibration = weight_monitor::load_calibration_from_file()
            .unwrap_or_else(|e| {
                warn!("Failed to load weight sensor calibration from file, will use default values instead. Error: {}", e);
                WeightSensorCalibration::default()
            });

        let (calibration_tx, calibration_rx) =
            tokio::sync::watch::channel(weight_sensor_calibration);

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
            version,
            power_sensor_mutex,
            power_readings_tx,
            power_readings_rx,
            weight_sensor_mutex,
            weight_readings_tx,
            weight_readings_rx,
            motor_cancel_token: None,
            calibration_in_progress: Arc::new(AtomicBool::new(false)),
            calibration_tx,
            calibration_rx,
        }
    }
}

fn init_weight_sensor(
    app_config: &AppConfig,
) -> Result<Box<dyn WeightSensor>, String> {
    match app_config.weight_monitor.sensor.as_str() {
        "SensorHX711" => {
            return Ok(Box::new(crate::sensors::sensor_hx711::SensorHx711::new(
                Bus::Spi0,
                SlaveSelect::Ss0,
            )?));
        }
        "SensorMock" => return Ok(Box::new(crate::sensors::sensor_mock::SensorMock::new())),
        _ => return Err(format!("Unsupported weight sensor type '{}'", app_config.weight_monitor.sensor)),
    };
}

fn init_power_sensor(
    app_config: &AppConfig,
) -> Result<Box<dyn PowerSensor>, String> {
    match app_config.power_monitor.sensor.as_str() {
        "SensorINA219" => return Ok(Box::new(crate::sensors::sensor_ina219::SensorIna219::new())),
        "SensorMock" => return Ok(Box::new(crate::sensors::sensor_mock::SensorMock::new())),
        _ => return Err(format!("Unsupported power sensor type '{}'", app_config.power_monitor.sensor)),
    };
}

fn init_motor(
    config: &AppConfig,
) -> Result<Box<dyn AsyncStepperMotor + Send + Sync>, String> {
    match config.motor.motor_type.as_str() {
        "Stepper28BYJ48" => Ok(Box::new(Stepper28BYJ48::new())),
        "StepperNema14" => {
            let nema14_config = match config.motor.nema14.clone() {
                Some(config) => config,
                None => return Err("Nema14 configuration is missing".to_string()),
            };
            Ok(Box::new(StepperNema14::new(nema14_config)))
        }
        "StepperMock" => Ok(Box::new(StepperMock::new())),
        _ => Err(format!("Unsupported motor type '{}'", config.motor.motor_type)),
    }
}
