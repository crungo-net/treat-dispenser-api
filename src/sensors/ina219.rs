use ina219::address::Address;
use ina219::calibration::IntCalibration;
use ina219::calibration::MicroAmpere;
use ina219::SyncIna219;
use linux_embedded_hal::I2cdev;
use tracing::{info, debug, error};

pub fn init_ina219_sensor() -> Result<SyncIna219<I2cdev, Option<IntCalibration>>, String> {
    info!("Initializing INA219 sensor");

    // Initialize the I2C device
    let i2c = I2cdev::new("/dev/i2c-1").map_err(|e| format!("Failed to initialize I2C device: {}", e))?;
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
            let error_msg = format!("Failed to create INA219 sensor at address {:#04X?}: {}", address_byte, e);
            error!("{}", error_msg.as_str());
            return Err(error_msg)
        },
    }

    let ina219 = ina219_init_result.unwrap();

    info!("INA219 sensor initialized successfully at address {}", address.as_byte());
    Ok(ina219)
}