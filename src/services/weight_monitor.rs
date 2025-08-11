use std::sync::{Arc, atomic::Ordering};
use std::time::Duration;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tokio::time::{interval, MissedTickBehavior};
use tracing::{info, error, debug, trace};
use crate::application_state::{self, ApplicationState};
use crate::sensors::{Calibration, WeightReading, WeightSensor};
use crate::utils::state_helpers;


pub async fn start_weight_monitoring_thread(
    app_state: Arc<Mutex<ApplicationState>>,
) {
    tokio::spawn({
        let app_state_clone = Arc::clone(&app_state);
        let sensor_mutex_opt = app_state_clone.lock().await.weight_sensor_mutex.clone();
        let weight_readings_tx = app_state_clone.lock().await.weight_readings_tx.clone();
        let calibration_in_progress = Arc::clone(&app_state_clone.lock().await.calibration_in_progress);

        async move {
            match sensor_mutex_opt {
                Some(sensor_mutex) => {
                    info!("Starting weight monitoring thread");

                    // If RATE=L (10 SPS): period ~100 ms. If RATE=H (80 SPS): ~12–15 ms.
                    let mut tick = interval(Duration::from_millis(100));
                    tick.set_missed_tick_behavior(MissedTickBehavior::Skip);

                    loop {
                        tick.tick().await;

                        if calibration_in_progress.load(Ordering::Relaxed) {
                            debug!("Calibration in progress, skipping weight reading");
                            continue;
                        }

                        let reading_result = {
                            let mut sensor = sensor_mutex.lock().await;
                            sensor.get_raw()
                        };

                        match reading_result {
                            Ok(weight) => {
                                trace!("Weight reading: {:?}", weight);
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


pub async fn calibrate_weight_sensor(
    app_state: Arc<Mutex<ApplicationState>>,
    known_mass_grams: f32,
) -> Result<(), String> {
    let app_state = Arc::clone(&app_state);

    let calibration_in_progress = app_state.lock().await.calibration_in_progress.clone();
    calibration_in_progress.store(true, Ordering::Relaxed);

    // Get the current calibration state
    let calibration_rx = app_state.lock().await.calibration_rx.clone();
    let calibration_tx = app_state.lock().await.calibration_tx.clone();
    let mut calibration = calibration_rx.borrow().clone();

    let sensor_mutex_opt = app_state.lock().await.weight_sensor_mutex.clone();
    let mut samples: Vec<WeightReading> = Vec::with_capacity(30);

    if let Some(sensor_mutex) = sensor_mutex_opt {
        // get approx 3 seconds of samples from weight sensor
        info!("Calibrating weight sensor, please wait...");

        for _ in 0..30 {
            let read_result = {
                let mut sensor = sensor_mutex.lock().await;
                sensor.get_raw()
            };
            match read_result {
                Ok(reading) => {
                    samples.push(reading);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                Err(e) => {
                    error!("Failed to read weight during tare: {}", e);
                }
            }
        }
    } else {
        return Err("No weight sensor available".to_string())
    }

    calibration_in_progress.store(false, Ordering::Relaxed);
    samples.sort_unstable();

    // calculate trimmed mean (trim off 20% from both ends)
    let k = (samples.len() as f32 * 0.2).round() as usize; // this is how many samples to trim from each end

    // subslice that excludes lowest k and highest k samples, 
    // ensuring we have at least one sample left after trimming
    let slice = &samples[k..samples.len().saturating_sub(k).max(k + 1)];
    let sum: i64 = slice.iter().map(|v| v.raw as i64).sum();
    let mean_raw = (sum as f32 / slice.len() as f32).round() as i32;

    // Calculate the scale factor
    let mut scale = (mean_raw - calibration.tare_raw) as f32 / known_mass_grams;
    if scale < 0.0 { scale = scale.abs();}

    calibration.scale = scale;
    let _ = calibration_tx.send(calibration.clone());
    Ok(())
}

pub async fn tare_weight_sensor(
    app_state: Arc<Mutex<ApplicationState>>,
) -> Result<TareResponse, String> {
    let app_state = Arc::clone(&app_state);

    let calibration_in_progress = app_state.lock().await.calibration_in_progress.clone();
    calibration_in_progress.store(true, Ordering::Relaxed);

    state_helpers::set_dispenser_status_async(
        &app_state,
        application_state::DispenserStatus::Calibrating,
    ).await;

    // Get the current calibration state
    let calibration_rx = app_state.lock().await.calibration_rx.clone();
    let calibration_tx = app_state.lock().await.calibration_tx.clone();
    let mut calibration = calibration_rx.borrow().clone();

    let sensor_mutex_opt = app_state.lock().await.weight_sensor_mutex.clone();
    let mut samples: Vec<WeightReading> = Vec::with_capacity(30);

    if let Some(sensor_mutex) = sensor_mutex_opt {
        // get approx 3 seconds of samples from weight sensor
        info!("Taring weight sensor, please wait...");

        for _ in 0..30 {
            let read_result = {
                let mut sensor = sensor_mutex.lock().await;
                sensor.get_raw()
            };
            match read_result {
                Ok(reading) => {
                    samples.push(reading);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                Err(e) => {
                    error!("Failed to read weight during tare: {}", e);
                }
            }
        }

    } else {
        calibration_in_progress.store(false, Ordering::Relaxed);
        state_helpers::set_dispenser_status_async(
            &app_state,
            application_state::DispenserStatus::CalibrationFailed,
        ).await;
        return Err("No weight sensor available".to_string());
    }

    calibration_in_progress.store(false, Ordering::Relaxed);
    samples.sort_unstable();

    // calculate trimmed mean (trim off 20% from both ends)
    let k = (samples.len() as f32 * 0.2).round() as usize; // this is how many samples to trim from each end

    // subslice that excludes lowest k and highest k samples, 
    // ensuring we have at least one sample left after trimming
    let slice = &samples[k..samples.len().saturating_sub(k).max(k + 1)];
    let sum: i64 = slice.iter().map(|v| v.raw as i64).sum();
    let tare_raw = (sum as f32 / slice.len() as f32).round() as i32;

    calibration.tare_raw = tare_raw;

    let calibration_publish_result = calibration_tx.send(calibration.clone());
    if calibration_publish_result.is_err() {
        error!("Failed to publish tare calibration");
        return Err("Failed to publish tare calibration".to_string());
    }

    info!("Tare completed, tare_raw: {}", tare_raw);

    state_helpers::set_dispenser_status_async(
        &app_state,
        application_state::DispenserStatus::Operational,
    ).await;

    Ok(TareResponse { msg: ("Tare successful.".to_string()), calibration })
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TareResponse {
    pub msg: String,
    pub calibration: Calibration,
}