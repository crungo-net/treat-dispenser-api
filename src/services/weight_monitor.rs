use crate::application_state::{self, ApplicationState};
use crate::sensors::{WeightSensorCalibration};
use crate::utils::state_helpers;
use crate::utils::filesystem;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, atomic::Ordering};
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::{MissedTickBehavior, interval};
use tracing::{debug, error, info, trace};

/// Spawns an asynchronous task that periodically reads the weight sensor (if present)
/// and publishes processed weight readings to subscribers. Skips sampling while a
/// calibration (tare or scale) operation is in progress.
///
/// * `app_state` - Shared application state containing sensor handles and channels.
pub async fn start_weight_monitoring_thread(app_state: Arc<Mutex<ApplicationState>>) {
    tokio::spawn({
        let app_state_clone = Arc::clone(&app_state);
        let sensor_mutex_opt = app_state_clone.lock().await.weight_sensor_mutex.clone();
        let weight_readings_tx = app_state_clone.lock().await.weight_readings_tx.clone();
        let calibration_in_progress =
            Arc::clone(&app_state_clone.lock().await.calibration_in_progress);

        let calibration_rx = app_state_clone.lock().await.calibration_rx.clone();

        async move {
            match sensor_mutex_opt {
                Some(sensor_mutex) => {
                    info!("Starting weight monitoring thread");

                    // If RATE=L (10 SPS): period ~100 ms. If RATE=H (80 SPS): ~12–15 ms.
                    let mut tick = interval(Duration::from_millis(15));
                    tick.set_missed_tick_behavior(MissedTickBehavior::Skip);

                    loop {
                        tick.tick().await;

                        if calibration_in_progress.load(Ordering::Relaxed) {
                            debug!("Calibration in progress, skipping weight reading");
                            continue;
                        }

                        let reading_result = {
                            let mut sensor = sensor_mutex.lock().await;
                            let calibration = calibration_rx.borrow().clone();
                            sensor.get_weight_reading(&calibration)
                        };

                        match reading_result {
                            Ok(weight) => {
                                trace!("Weight reading: {:?}", weight);
                                let _ = weight_readings_tx.send(weight);
                            }
                            Err(e) => {
                                trace!("Failed to read weight: {}", e);
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

/// Performs a scale calibration using a known mass placed on the load cell.
/// Collects a fixed number of raw samples, computes a trimmed mean, and derives a
/// scale factor relative to the previously stored tare value.
///
/// * `app_state` - Shared application state.
/// * `known_mass_grams` - Mass (in grams) of the calibration weight currently on the platform.
///
/// Returns updated calibration metadata (including new scale factor) or an error.
pub async fn calibrate_weight_sensor(
    app_state: Arc<Mutex<ApplicationState>>,
    known_mass_grams: f32,
) -> Result<CalibrationResponse, String> {
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
    let mut samples: Vec<i32> = Vec::with_capacity(300);

    if let Some(sensor_mutex) = sensor_mutex_opt {
        // get approx 3 seconds of samples from weight sensor
        info!("Calibrating weight sensor, please wait...");

        for _ in 0..300 {
            let read_result = {
                let mut sensor = sensor_mutex.lock().await;
                sensor.get_raw()
            };
            match read_result {
                Ok(reading) => {
                    samples.push(reading);
                    tokio::time::sleep(Duration::from_millis(15)).await;
                }
                Err(e) => {
                    trace!("Failed to read weight during calibration: {}", e);
                }
            }
        }
    } else {
        state_helpers::set_dispenser_status_async(
            &app_state,
            application_state::DispenserStatus::CalibrationFailed,
        ).await;
        return Err("No weight sensor available".to_string());
    }

    calibration_in_progress.store(false, Ordering::Relaxed);

    let mean_raw = calculate_trimmed_mean(&mut samples);

    // Calculate the scale factor
    let mut scale = (mean_raw - calibration.tare_raw as f32) / known_mass_grams;
    if scale < 0.0 {
        scale = scale.abs();
    }

    calibration.scale = scale;
    let _ = calibration_tx.send(calibration.clone());

    // save the updated calibration to file
    if let Err(e) = save_calibration_to_file(&calibration) {
        error!("Failed to save calibration to file: {}", e);
    }

    state_helpers::set_dispenser_status_async(
        &app_state,
        application_state::DispenserStatus::Operational,
    ).await;

    Ok(CalibrationResponse {
        msg: format!("Calibration successful. Scale factor: {:.4}", scale),
        calibration,
    })
}

/// Performs a tare (zero) calibration. Samples the load cell with no weight applied,
/// computes a trimmed mean, and stores it as the new tare baseline in shared state.
///
/// * `app_state` - Shared application state.
///
/// Returns updated calibration metadata including the new tare value or an error.
pub async fn tare_weight_sensor(
    app_state: Arc<Mutex<ApplicationState>>,
) -> Result<CalibrationResponse, String> {
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
    let mut samples: Vec<i32> = Vec::with_capacity(300);

    if let Some(sensor_mutex) = sensor_mutex_opt {
        // get approx 3 seconds of samples from weight sensor
        info!("Taring weight sensor, please wait...");

        for _ in 0..300 {
            let read_result = {
                let mut sensor = sensor_mutex.lock().await;
                sensor.get_raw()
            };
            match read_result {
                Ok(reading) => {
                    samples.push(reading);
                    tokio::time::sleep(Duration::from_millis(15)).await;
                }
                Err(e) => {
                    trace!("Failed to read weight during tare: {}", e);
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
    let tare_raw = calculate_trimmed_mean(&mut samples);

    calibration.tare_raw = tare_raw as i32;

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

    Ok(CalibrationResponse {
        msg: ("Tare successful.".to_string()),
        calibration,
    })
}

/// Response returned by calibration/tare endpoints containing a human-friendly
/// message and the updated calibration state.
#[derive(Clone, Debug, Serialize)]
pub struct CalibrationResponse {
    pub msg: String,
    pub calibration: WeightSensorCalibration,
}

/// Request payload for scale calibration; carries the known mass (in grams)
/// currently placed on the load cell.
#[derive(Deserialize)]
pub struct CalibrationRequest {
    pub known_mass_grams: f32,
}

/// Computes a 20% trimmed mean (removes the lowest and highest 20% of values)
/// from the supplied sample slice, returning a rounded f32. Helps reject outliers
/// and reduce noise in raw load cell readings.
fn calculate_trimmed_mean(samples: &mut [i32]) -> f32 {
    samples.sort_unstable();

    let k = (samples.len() as f32 * 0.2).round() as usize; 

    // subslice that excludes lowest k and highest k samples,
    // ensuring we have at least one sample left after trimming
    let slice = &samples[k..samples.len().saturating_sub(k).max(k + 1)];
    let sum: i64 = slice.iter().map(|v| *v as i64).sum();
    let trimmed_mean = (sum as f32 / slice.len() as f32).round();

    trimmed_mean
}

pub fn save_calibration_to_file(
    calibration: &WeightSensorCalibration,
) -> Result<(), String> {
    filesystem::save_json_to_file(&filesystem::get_calibration_file_path(), calibration)
        .map_err(|e| format!("Failed to save calibration to file: {}", e))
}


pub fn load_calibration_from_file() -> Result<WeightSensorCalibration, String> {
    filesystem::read_json_from_file(&filesystem::get_calibration_file_path())
        .map_err(|e| format!("Failed to read calibration from file: {}", e))
}