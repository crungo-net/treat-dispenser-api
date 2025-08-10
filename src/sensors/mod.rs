pub mod sensor_ina219;
pub mod sensor_mock;
pub mod sensor_hx711;

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

#[derive(Clone, Debug)]
pub struct WeightReading {
    pub weight_grams: f32,
}

impl WeightReading {
    pub fn dummy() -> Self {
        WeightReading {
            weight_grams: -1.0,
        }
    }
}

impl Default for WeightReading {
    fn default() -> Self {
        WeightReading {
            weight_grams: 0.0,
        }
    }
}

pub trait PowerSensor: Send + Sync {
    fn get_name(&self) -> String;
    fn get_power_reading(&mut self) -> Result<PowerReading, String>;
}

pub trait WeightSensor {
    fn get_name(&self) -> String;
    fn get_weight(&mut self) -> Result<WeightReading, String>;
}