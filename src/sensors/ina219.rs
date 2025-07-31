use ina219::address::Address;
use ina219::calibration::UnCalibrated;
use ina219::SyncIna219;
use linux_embedded_hal::I2cdev;
use tracing::{info, debug};

pub fn init_ina219_sensor() -> Result<SyncIna219<I2cdev, UnCalibrated>, String> {
    info!("Initializing INA219 sensor");

    // Initialize the I2C device
    let i2c = I2cdev::new("/dev/i2c-1").map_err(|e| format!("Failed to initialize I2C device: {}", e))?;
    debug!("I2C device initialized");

    let address = Address::from_byte(0x40).unwrap();
    debug!("Using I2C address: {}", address.as_byte());
    
    // Create a new INA219 sensor instance
    let ina219 = SyncIna219::new(i2c, address).unwrap();
        //.map_err(|e| format!("Failed to create INA219 sensor: {}", e))?;
    debug!("INA219 instance created at address {}", address.as_byte());

    let result = ina219;

    std::thread::sleep(std::time::Duration::from_millis(5000)); // Allow time for sensor to stabilize
    info!("INA219 sensor initialized successfully at address {}", address.as_byte());
    Ok(result)
}