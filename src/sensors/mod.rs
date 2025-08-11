use serde::{Deserialize, Serialize};

pub mod sensor_hx711;
pub mod sensor_ina219;
pub mod sensor_mock;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeightSensorCalibration {
    /// Scale factor for converting raw readings to grams
    pub scale: f32,

    /// Offset to apply to raw readings
    pub offset: f32,

    /// Raw tare value to subtract from readings
    pub tare_raw: i32,
}

impl Default for WeightSensorCalibration {
    fn default() -> Self {
        WeightSensorCalibration {
            scale: 1.0,
            offset: 0.0,
            tare_raw: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PowerReading {
    pub bus_voltage_volts: f32,
    pub current_amps: f32,
    pub power_watts: f32,
}

impl PowerReading {
    pub fn dummy() -> Self {
        PowerReading {
            bus_voltage_volts: -1.0,
            current_amps: -1.0,
            power_watts: -1.0,
        }
    }
}

impl Default for PowerReading {
    fn default() -> Self {
        PowerReading {
            bus_voltage_volts: 0.0,
            current_amps: 0.0,
            power_watts: 0.0,
        }
    }
}

#[derive(Clone, Debug, Ord, PartialEq, Eq, PartialOrd)]
pub struct WeightReading {
    pub grams: i32,
}

impl WeightReading {
    pub fn dummy() -> Self {
        WeightReading { grams: -1 }
    }
}

impl Default for WeightReading {
    fn default() -> Self {
        WeightReading { grams: 0 }
    }
}

pub trait PowerSensor: Send + Sync {
    fn get_name(&self) -> String;
    fn get_power_reading(&mut self) -> Result<PowerReading, String>;
}

pub trait WeightSensor: Send {
    fn get_name(&self) -> String;
    fn get_weight_reading(
        &mut self,
        calibration: &WeightSensorCalibration,
    ) -> Result<WeightReading, String>;
    fn get_raw(&mut self) -> Result<i32, String>;
}
