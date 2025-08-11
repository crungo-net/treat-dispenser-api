pub fn get_config_path() -> String {
    "/etc/treat-dispenser-api/config.yaml".to_string() // todo: make this configurable
}

pub fn save_json_to_file<T: serde::Serialize>(path: &str, data: &T) -> Result<(), String> {
    let json_data = serde_json::to_string(data).map_err(|e| e.to_string())?;
    std::fs::write(path, json_data).map_err(|e| e.to_string())
}

pub fn read_json_from_file<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, String> {
    let json_data = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&json_data).map_err(|e| e.to_string())
}