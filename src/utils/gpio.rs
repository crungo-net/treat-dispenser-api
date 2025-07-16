use rppal::gpio::{Gpio, OutputPin, Result};

pub const STEPPER_PINS: [u8; 4] = [26, 19, 13, 6];

pub fn init_stepper_pins(gpio: &Gpio) -> Result<Vec<OutputPin>> {
    STEPPER_PINS
        .iter()
        .map(|&pin| gpio.get(pin).map(|p| p.into_output()))
        .collect()
}
