use crate::sensors::PowerReading;
use crate::sensors::PowerSensor;
use crate::sensors::WeightSensor;
use crate::sensors::WeightSensorCalibration;

pub struct SensorMock {}

impl WeightSensor for SensorMock {
    fn get_name(&self) -> String {
        "SensorMock".to_string()
    }

    fn get_weight_reading(
        &mut self,
        _calibration: &WeightSensorCalibration,
    ) -> Result<crate::sensors::WeightReading, String> {
        // Return a dummy weight reading for testing purposes
        Ok(crate::sensors::WeightReading { grams: 12345 })
    }

    fn get_raw(&mut self) -> Result<i32, String> {
        // Return a dummy raw reading for testing purposes
        Ok(123456)
    }
}

impl PowerSensor for SensorMock {
    fn get_name(&self) -> String {
        "SensorMock".to_string()
    }

    fn get_power_reading(&mut self) -> Result<PowerReading, String> {
        // Return a dummy power reading for testing purposes
        Ok(PowerReading {
            bus_voltage_volts: 12.0,
            current_amps: 0.6,
            power_watts: 0.5,
        })
    }
}

impl SensorMock {
    pub fn new() -> Self {
        SensorMock {}
    }
}
