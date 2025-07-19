use crate::utils::filesystem::get_nema14_config_path;
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nema14Config {
    pub dir_pin: u8,
    pub step_pin: u8,
    pub sleep_pin: u8,
    pub reset_pin: u8,
    pub enable_pin: u8,
}

impl Default for Nema14Config {
    fn default() -> Self {
        Nema14Config {
            dir_pin: 26,
            step_pin: 19,
            sleep_pin: 13,
            reset_pin: 6,
            enable_pin: 17,
        }
    }
}

pub fn load_nema14_config_from_str(config_str: &str) -> Result<Nema14Config, String> {
    serde_yaml::from_str(config_str).map_err(|e| format!("Failed to parse config: {}", e))
}

pub fn load_nema14_config() -> Result<Nema14Config, String> {
    let config_path = get_nema14_config_path();
    let config_str_result = std::fs::read_to_string(&config_path);

    match config_str_result {
        Ok(config_str) => load_nema14_config_from_str(config_str.as_str()),
        Err(e) => {
            warn!(
                "Failed to read Nema14 config file (path: {}): {}",
                config_path, e
            );
            return Err(format!("Failed to read Nema14 config: {}", e));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_nema14_config_from_str() {
        let config_str = r#"
            dir_pin: 26
            step_pin: 19
            sleep_pin: 13
            reset_pin: 6
            enable_pin: 17
        "#;

        let config = load_nema14_config_from_str(config_str).unwrap();

        assert_eq!(config.dir_pin, 26);
        assert_eq!(config.step_pin, 19);
        assert_eq!(config.sleep_pin, 13);
        assert_eq!(config.reset_pin, 6);
        assert_eq!(config.enable_pin, 17);
    }
}
