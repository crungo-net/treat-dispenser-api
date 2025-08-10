use std::thread;
use std::time::Duration;

use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use hx711_spi::{Hx711, Hx711Error, Mode as HxMode};
use crate::sensors::WeightReading;
use crate::sensors::WeightSensor;
use tracing::{info, error};


type HxError = Hx711Error<rppal::spi::Error>;

pub struct SensorHx711 {
    hx711: Hx711<Spi>,
}

impl SensorHx711 {
    pub fn new(_spi_bus: Bus, _slave_select: SlaveSelect) -> Result<Self, String> {
        let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode1).unwrap();
        
        let mut hx711 = Hx711::new(spi);

        match hx711.reset() {
            Ok(_) => (),
            Err(e) => return Err(format!("Failed to reset HX711: {:?}", e)),
        }

        match hx711.set_mode(HxMode::ChAGain128) {
            Ok(_) => (),
            Err(e) => return Err(format!("Failed to set HX711 mode: {:?}", e)),
        }

        Ok(SensorHx711 { hx711 })
    }
}

impl WeightSensor for SensorHx711 {
    fn get_name(&self) -> String {
        "HX711".to_string()
    }

    fn get_weight(&mut self) -> Result<WeightReading, String> {
        let hx711 = &mut self.hx711;
        for _ in 0..20 {
            let read_result = hx711.read(); // 24-bit two's-complement, sign-extended
            let raw = match read_result {
                Ok(value) => value,
                Err(e) => {
                    error!("Failed to read from HX711: {:?}", e);
                    return Err(format!("HX711 read error: {:?}", e));
                }
            };
            info!("raw={raw}");
            thread::sleep(Duration::from_millis(100)); // ~10 SPS is plenty for testing
        }
        let reading = WeightReading {
            weight_grams: 0.0, // Replace with actual conversion logic if needed
        };
        Ok(reading)
    }
}