
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use hx711_spi::{Hx711, Hx711Error, Mode as HxMode};
use crate::sensors::WeightReading;
use crate::sensors::WeightSensor;
use tracing::{info, error, trace};


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

        // Poll until ready (read returns WouldBlock while not ready)
        loop {
            match hx711.set_mode(HxMode::ChAGain128) {
                Ok(_) => break, // success; mode applied for the next conversion
                Err(Hx711Error::DataNotReady) => std::thread::sleep(std::time::Duration::from_millis(20)),
                Err(e) => return Err(format!("Failed to set HX711 mode: {:?}", e)),
            }
        }

        Ok(SensorHx711 { hx711 })
    }
}

impl WeightSensor for SensorHx711 {
    fn get_name(&self) -> String {
        "HX711".to_string()
    }

    fn get_raw(&mut self) -> Result<WeightReading, String> {
        let hx711 = &mut self.hx711;
        let read_result = hx711.read(); // 24-bit two's-complement, sign-extended
        let raw = match read_result {
            Ok(value) => value,
            Err(e) => {
                return Err(format!("HX711 read error: {:?}", e));
            }
        };
        trace!("raw={raw}");
        let reading = WeightReading {
            raw, 
        };
        Ok(reading)
    }
}