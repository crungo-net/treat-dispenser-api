pub mod sensor_ina219;
pub mod sensor_mock;

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

pub trait PowerSensor: Send + Sync {
    fn get_name(&self) -> String;
    fn get_power_reading(&mut self) -> Result<PowerReading, String>;
}