use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::application_state;

pub async fn start_power_monitoring_thread(
    app_state: Arc<Mutex<application_state::ApplicationState>>,
) {
    tokio::spawn({
        let current_sensor = app_state.lock().await.current_sensor.clone();
        let power_readings_tx = app_state.lock().await.power_readings_tx.clone();

        async move {
            info!("Starting power monitoring thread");
            loop {
                match &current_sensor {
                    Some(sensor_mutex) => {
                        let power_reading_result = sensor_mutex.lock().await.get_power_reading();

                        match power_reading_result {
                            Ok(power_reading) => {
                                // publish the power reading to the channel
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
            }
        }
    });
}
