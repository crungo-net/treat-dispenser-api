use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, error, warn, info};

use crate::application_state;
use crate::sensors::PowerReading;

struct PowerMonitor {
    readings_vec: Vec<PowerReading>,
}

impl PowerMonitor {
    pub fn new() -> Self {
        PowerMonitor {
            readings_vec: Vec::new(),
        }
    }

    pub fn add_reading(&mut self, reading: PowerReading) {
        self.readings_vec.push(reading);
    }

    pub fn get_readings(&self) -> &Vec<PowerReading> {
        &self.readings_vec
    }

    pub fn clear_readings(&mut self) {
        self.readings_vec.clear();
    }

    pub fn get_average_power(&self) -> f32 {
        if self.readings_vec.is_empty() {
            return 0.0;
        }
        let total: f32 = self.readings_vec.iter().map(|r| r.power_watts).sum();
        total / self.readings_vec.len() as f32
    }

    pub fn get_average_current(&self) -> f32 {
        if self.readings_vec.is_empty() {
            return 0.0;
        }
        let total: f32 = self.readings_vec.iter().map(|r| r.current_amps).sum();
        total / self.readings_vec.len() as f32
    }
}

pub async fn start_power_monitoring_thread(
    app_state: Arc<Mutex<application_state::ApplicationState>>,
) {
    tokio::spawn({
        let current_sensor = app_state.lock().await.power_sensor_mutex.clone();
        let power_readings_tx = app_state.lock().await.power_readings_tx.clone();
        let app_state_clone = Arc::clone(&app_state);

        async move {
            info!("Starting power monitoring thread");
            let mut power_monitor = PowerMonitor::new();
            let mut i = 0;
            loop {
                // log and clear power readings after every 30 readings (approx every 3 seconds)
                if i == 30 {
                    let avg_current = power_monitor.get_average_current();
                    debug!(
                        "Average current over last {} readings: {} A",
                        power_monitor.get_readings().len(),
                        avg_current
                    );

                    // if average current is above 0.7 (default) amps, log it and cancel ongoing motor operations
                    let config = app_state_clone.lock().await.app_config.clone();
                    let current_limit = config.motor_current_limit_amps.unwrap_or(0.7);

                    if avg_current > current_limit {
                        warn!("High average current detected: {} A", avg_current);
                        let state_guard = app_state_clone.lock().await;

                        if let Some(cancel_token) = &state_guard.motor_cancel_token {
                            info!("Cancelling ongoing motor operations due to high current.");
                            cancel_token.cancel();
                        }
                    }
                    power_monitor.clear_readings();
                    i = 0;
                }

                match &current_sensor {
                    Some(sensor_mutex) => {
                        let power_reading_result = sensor_mutex.lock().await.get_power_reading();

                        match power_reading_result {
                            Ok(power_reading) => {
                                // publish the power reading to the channel
                                power_monitor.add_reading(power_reading.clone());
                                let _ = power_readings_tx.send(power_reading);
                            }
                            Err(e) => {
                                error!("Failed to get power reading: {}", e);
                            }
                        }
                    }
                    None => {
                        error!("Power monitor is not initialized");
                        break;
                    }
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
                i += 1;
            }
        }
    });
}


#[cfg(test)]
mod tests {
    use super::*; 

    #[test]
    fn test_calculate_average_power_and_current() {
        let mut monitor = PowerMonitor::new();
        monitor.add_reading(PowerReading {
            bus_voltage_volts: 12.0,
            current_amps: 1.0,
            power_watts: 12.0,
        });
        monitor.add_reading(PowerReading {
            bus_voltage_volts: 12.0,
            current_amps: 2.0,
            power_watts: 24.0,
        }); 
        assert_eq!(monitor.get_average_power(), 18.0);
        assert_eq!(monitor.get_average_current(), 1.5);
    }

    #[test]
    fn test_clear_readings() {
        let mut monitor = PowerMonitor::new();
        monitor.add_reading(PowerReading {
            bus_voltage_volts: 12.0,
            current_amps: 1.0,
            power_watts: 12.0,
        });
        assert!(!monitor.get_readings().is_empty());
        monitor.clear_readings();
        assert!(monitor.get_readings().is_empty());
    }
}