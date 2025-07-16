use chrono::{DateTime, Local};
use std::time::SystemTime;

/// Converts a SystemTime to a formatted string in the local timezone
/// in the format "YYYY-MM-DD HH:MM:SS".
pub fn format_system_time(system_time: SystemTime) -> String {
    let datetime: DateTime<Local> = system_time.into();
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Returns the current timestamp formatted as a string in the local timezone
/// in the format "YYYY-MM-DD HH:MM:SS".
pub fn get_formatted_current_timestamp() -> String {
    let now = SystemTime::now();
    format_system_time(now)
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn test_format_system_time() {
        // Create a known time: 2023-01-01 12:00:00 UTC
        let test_time = UNIX_EPOCH + Duration::from_secs(1672574400);

        // Format the time
        let formatted = format_system_time(test_time);

        // We need to be timezone-aware in our test, so we'll check the pattern
        // rather than the exact string to make the test reliable across different timezones
        let re = Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$").unwrap();
        assert!(
            re.is_match(&formatted),
            "Formatted time doesn't match expected pattern: {}",
            formatted
        );
    }

    #[test]
    fn test_get_formatted_current_timestamp() {
        // Get the formatted current timestamp
        let timestamp = get_formatted_current_timestamp();

        // Verify the format
        let re = Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$").unwrap();
        assert!(
            re.is_match(&timestamp),
            "Current timestamp doesn't match expected pattern: {}",
            timestamp
        );
    }
}
