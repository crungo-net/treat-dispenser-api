use std::sync::{Arc};
use std::time::Duration;
use rppal::spi::{Bus, SlaveSelect};
use tokio::sync::Mutex;
use tokio::time::{interval, MissedTickBehavior};
use tracing::{info, error};
use crate::application_state::ApplicationState;
use crate::sensors::{WeightReading, WeightSensor};


pub async fn start_weight_monitoring_thread(
    app_state: Arc<Mutex<ApplicationState>>,
) {
    tokio::spawn({
        let app_state_clone = Arc::clone(&app_state);
        let sensor_mutex_opt = app_state_clone.lock().await.weight_sensor_mutex.clone();
        let weight_readings_tx = app_state_clone.lock().await.weight_readings_tx.clone();

        async move {
            match sensor_mutex_opt {
                Some(sensor_mutex) => {
                    info!("Starting weight monitoring thread");

                    // If RATE=L (10 SPS): period ~100 ms. If RATE=H (80 SPS): ~12–15 ms.
                    let mut tick = interval(Duration::from_millis(100));
                    tick.set_missed_tick_behavior(MissedTickBehavior::Skip);

                    loop {
                        tick.tick().await;

                        let reading_result = {
                            let mut sensor = sensor_mutex.lock().await;
                            sensor.get_weight()
                        };

                        match reading_result {
                            Ok(weight) => {
                                info!("Weight reading: {:?}", weight);
                                let _ = weight_readings_tx.send(weight);
                            }
                            Err(e) => {
                                error!("Failed to read weight: {}", e);
                            }
                        }
                    }
                }
                None => {
                    error!("No weight sensor available");
                }
            }
        }
    });
}