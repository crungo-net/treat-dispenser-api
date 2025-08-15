use crate::utils;
use crate::motor::stepper_nema14::Nema14Config;

use tracing ::{debug};

pub const MOTOR_COOLDOWN_MS_DEFAULT: u64 = 5000;
pub const MOTOR_CURRENT_LIMIT_AMPS_DEFAULT: f32 = 0.7;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct ApiConfig {
    pub listen_address: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct PowerMonitorConfig {
    pub sensor: String,
    pub motor_current_limit_amps: Option<f32>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct WeightMonitorConfig {
    pub sensor: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct MotorConfig {
    pub motor_type: String,
    pub nema14: Option<Nema14Config>,
    pub cooldown_ms: Option<u64>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct AppConfig {
    pub api: ApiConfig,
    pub nema14: Option<Nema14Config>,
    pub motor: MotorConfig,
    pub power_monitor: PowerMonitorConfig,
    pub weight_monitor: WeightMonitorConfig,
    pub admin_user: String,
    pub admin_password: String,
}

pub fn load_app_config_from_str(config_str: &str) -> AppConfig {
    serde_yaml::from_str(config_str).expect("Failed to parse app config")
}

pub fn load_app_config() -> AppConfig {
    let app_config_path = utils::filesystem::get_config_path();
    let config_str = std::fs::read_to_string(&app_config_path).expect(&format!(
        "Failed to read app config file at {}",
        app_config_path
    ));

    let app_config: AppConfig = load_app_config_from_str(&config_str);

    // Log the config struct as json
    debug!(
        "Parsed app config: {}",
        serde_json::to_string(&app_config).unwrap_or_default()
    );
    app_config
}