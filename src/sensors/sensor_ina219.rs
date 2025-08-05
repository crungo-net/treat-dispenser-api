use ina219::SyncIna219;
use ina219::address::Address;
use ina219::calibration::IntCalibration;
use ina219::calibration::MicroAmpere;
use linux_embedded_hal::I2cdev;
use tracing::{debug, error, warn, info};
use crate::sensors::PowerReading;
use crate::sensors::PowerSensor;

fn init_ina219_sensor() -> Result<SyncIna219<I2cdev, Option<IntCalibration>>, String> {
    info!("Initializing INA219 sensor");

    // Initialize the I2C device
    let i2c =
        I2cdev::new("/dev/i2c-1").map_err(|e| format!("Failed to initialize I2C device: {}", e))?;
    debug!("I2C device initialized");

    let address_byte = 0x40; // Default I2C address for INA219, todo: make configurable
    let address = Address::from_byte(0x40).unwrap();
    debug!("Using default I2C address: {:#04X}", address_byte);

    // Create a new INA219 sensor instance
    // Calibrate with resolution of 1A, and a shunt of 100 milliohms (0.1 ohm)
    let calibration = IntCalibration::new(MicroAmpere(1_000_000), 1_00);
    let ina219_init_result = SyncIna219::new_calibrated(i2c, address, calibration);

    match ina219_init_result {
        Ok(_) => info!("INA219 sensor created successfully"),
        Err(e) => {
            let error_msg = format!(
                "Failed to create INA219 sensor at address {:#04X?}: {}",
                address_byte, e
            );
            error!("{}", error_msg.as_str());
            return Err(error_msg);
        }
    }

    let ina219 = ina219_init_result.unwrap();

    info!(
        "INA219 sensor initialized successfully at address {}",
        address.as_byte()
    );
    Ok(ina219)
}

pub struct SensorIna219 {
    ina219: SyncIna219<I2cdev, Option<IntCalibration>>,
}

impl SensorIna219 {
    pub fn new() -> Self {
        let ina219 = init_ina219_sensor().unwrap_or_else(|e| {
            error!("Failed to initialize INA219 sensor: {}", e);
            panic!("INA219 sensor initialization failed");
        });
        SensorIna219 { ina219 }
    }

    pub fn get_bus_voltage(&mut self) -> Result<f32, String> {
        let bus_voltage = self
            .ina219
            .bus_voltage()
            .map_err(|e| format!("Failed to read bus voltage: {}", e))?;
        Ok(bus_voltage.voltage_mv() as f32 / 1000.0) // Convert mV to V
    }

    pub fn get_current_amps(&mut self) -> Result<f32, String> {
        let current = self
            .ina219
            .current_raw()
            .map_err(|e| format!("Failed to read current: {}", e))?;

        let current_amps = current.0 as f32 / 1000.0; // Convert mA to A
        if current_amps > 2.0 {
            warn!("Current reading is unrealistic: {} A", current_amps);
        }
        Ok(current_amps.clamp(0.0, 2.0)) // clamped to realistic range
    }

}

impl PowerSensor for SensorIna219 {
    fn get_name(&self) -> String {
        "SensorINA219".to_string()
    }

    fn get_power_reading(&mut self) -> Result<PowerReading, String> {
        let bus_voltage = self.get_bus_voltage()?;
        let current = self.get_current_amps()?;
        let power = bus_voltage * current;

        Ok(PowerReading {
            bus_voltage_volts: bus_voltage,
            current_amps: current,
            power_watts: power,
        })
    }
}