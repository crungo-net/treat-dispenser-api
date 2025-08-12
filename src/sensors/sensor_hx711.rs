use crate::sensors::WeightReading;
use crate::sensors::WeightSensor;
use crate::sensors::WeightSensorCalibration;
use hx711_spi::{Hx711, Hx711Error, Mode as HxMode};
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use tracing::{info};

pub struct SensorHx711 {
    hx711: Hx711<Spi>,
}

impl SensorHx711 {
    pub fn new(_spi_bus: Bus, _slave_select: SlaveSelect) -> Result<Self, String> {
        let spi_result = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode1);

        let spi = match spi_result {
            Ok(s) => s,
            Err(e) => return Err(format!("Failed to initialize SPI: {:?}", e)),
        };

        let mut hx711 = Hx711::new(spi);

        match hx711.reset() {
            Ok(_) => (),
            Err(e) => return Err(format!("Failed to reset HX711: {:?}", e)),
        }

        // Poll until ready (read returns WouldBlock while not ready)
        loop {
            match hx711.set_mode(HxMode::ChAGain128) {
                Ok(_) => break, // success; mode applied for the next conversion
                Err(Hx711Error::DataNotReady) => {
                    std::thread::sleep(std::time::Duration::from_millis(20))
                }
                Err(e) => return Err(format!("Failed to set HX711 mode: {:?}", e)),
            }
        }

        info!("Initialized HX711 on SPI bus {:?} with slave select {:?}", _spi_bus, _slave_select);
        Ok(SensorHx711 { hx711 })
    }
}

impl WeightSensor for SensorHx711 {
    fn get_name(&self) -> String {
        "SensorHX711".to_string()
    }

    fn get_weight_reading(
        &mut self,
        calibration: &WeightSensorCalibration,
    ) -> Result<WeightReading, String> {
        let read_result = self.get_raw();
        let raw = match read_result {
            Ok(value) => value,
            Err(e) => {
                return Err(format!("Could not get weight reading in grams: {:?}", e));
            }
        };

        let mut grams = SensorHx711::grams_from_raw(raw, &calibration).round() as i32;

        //trace!("grams={grams}");
        if grams.abs() < 1 { 
            grams = 0; 
        } // 1 g deadband

        let reading = WeightReading { grams };
        Ok(reading)
    }

    fn get_raw(&mut self) -> Result<i32, String> {
        let hx711 = &mut self.hx711;
        let read_result = hx711.read(); // 24-bit two's-complement, sign-extended
        let raw = match read_result {
            Ok(value) => value,
            Err(e) => {
                return Err(format!("HX711 read error: {:?}", e));
            }
        };
        //trace!("raw={raw}");
        Ok(raw)
    }
}

impl SensorHx711 {
    fn grams_from_raw(raw: i32, cal: &WeightSensorCalibration) -> f32 {
        ((raw as f32 - cal.tare_raw as f32) - cal.offset) / cal.scale
    }
}
