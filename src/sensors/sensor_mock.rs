use crate::sensors::PowerSensor;
use crate::sensors::PowerReading;

pub struct SensorMock {}

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
