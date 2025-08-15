use crate::utils;
use crate::motor::stepper_nema14::Nema14Config;

use tracing ::{debug};

pub const MOTOR_COOLDOWN_MS_DEFAULT: u64 = 5000;
pub const MOTOR_CURRENT_LIMIT_AMPS_DEFAULT: f32 = 0.7;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct ApiConfig {
    pub listen_address: String,
    pub admin_user: String,
    pub admin_password: String,
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
    pub motor: MotorConfig,
    pub power_monitor: PowerMonitorConfig,
    pub weight_monitor: WeightMonitorConfig,
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

#[cfg(test)]
mod tests {
    use super::*;   

    #[test]
    fn test_load_app_config() {
        let config_str = r#"
        api:
            listen_address: "0.0.0.0:3500"
            admin_user: "admin"
            admin_password: "password"

        motor:
            motor_type: "StepperNema14"
            cooldown_ms: 5000
            nema14:
              dir_pin: 26
              step_pin: 19
              sleep_pin: 13
              reset_pin: 6
              enable_pin: 17

        power_monitor:
            sensor: "SensorINA219"
            motor_current_limit_amps: 0.7

        weight_monitor:
            sensor: "SensorHX711"
        "# ;  

        let config = load_app_config_from_str(config_str);
        let nema14_config_opt = config.motor.nema14.clone();

        assert_eq!(config.api.listen_address, "0.0.0.0:3500");
        assert_eq!(config.api.admin_user, "admin");
        assert_eq!(config.api.admin_password, "password");
        assert_eq!(config.motor.motor_type, "StepperNema14");
        assert_eq!(config.motor.cooldown_ms, Some(5000));
        assert_eq!(config.power_monitor.sensor, "SensorINA219");
        assert_eq!(config.power_monitor.motor_current_limit_amps, Some(0.7));
        assert_eq!(config.weight_monitor.sensor, "SensorHX711");

        assert!(nema14_config_opt.is_some());
        let nema14_config = nema14_config_opt.unwrap();

        assert_eq!(nema14_config.dir_pin, 26);
        assert_eq!(nema14_config.step_pin, 19);
        assert_eq!(nema14_config.sleep_pin, 13);
        assert_eq!(nema14_config.reset_pin, 6);
        assert_eq!(nema14_config.enable_pin, 17);

    }
}