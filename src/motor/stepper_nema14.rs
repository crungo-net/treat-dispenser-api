use crate::motor::{Direction, StepMode, StepperMotor};

use rppal::gpio::{Gpio, OutputPin};
use std::time::Duration;
use tracing::info;

pub struct StepperNema14 {}

impl StepperMotor for StepperNema14 {
    fn get_name(&self) -> String {
        "StepperNema14".to_string()
    }

    fn get_step_count_for_full_rotation(&self, _step_mode: &StepMode) -> u32 {
        200
    }

    fn run_motor(
        &self,
        steps: u32,
        direction: &Direction,
        step_mode: &StepMode,
    ) -> Result<u32, String> {
        info!("Starting NEMA14 motor with {} steps", steps);

        match step_mode {
            StepMode::Full => {
                // NEMA14 typically supports full and half step modes
                info!("Using {} step mode", step_mode);
            }
            _ => {
                return Err("Unsupported step mode for NEMA14".to_string());
            }
        }

        match Gpio::new() {

            Ok(_gpio) => {
                let mut step_pin = self.get_step_pin()?;
                let mut dir_pin = self.get_direction_pin()?;
                let mut _sleep_pin = self.get_sleep_pin()?;
                let mut _reset_pin = self.get_reset_pin()?;
                
                //sleep_pin.write(rppal::gpio::Level::High);
                //reset_pin.write(rppal::gpio::Level::High);
                let mut enable_pin = self.get_enable_pin()?;
                enable_pin.write(rppal::gpio::Level::Low); // Enable the motor

                match direction {
                    Direction::Clockwise => dir_pin.write(rppal::gpio::Level::High),
                    Direction::CounterClockwise => dir_pin.write(rppal::gpio::Level::Low),
                }

                for _ in 0..steps {
                    step_pin.write(rppal::gpio::Level::High);
                    std::thread::sleep(Duration::from_micros(1000));
                    step_pin.write(rppal::gpio::Level::Low);
                    std::thread::sleep(Duration::from_micros(1000));
                }

                enable_pin.write(rppal::gpio::Level::High);
                Ok(steps)
            }
            Err(e) => {
                return Err(format!("Failed to initialize GPIO: {}", e));
            }
        }
    }

    fn run_motor_degrees(
            &self,
            degrees: f32,
            direction: &Direction,
            step_mode: &StepMode,
        ) -> Result<u32, String> {
        
        self.run_motor((degrees / 1.80) as u32, direction, step_mode)
    }
}

impl StepperNema14 {
    pub fn new() -> Self {
        StepperNema14 {}
    }

    pub fn get_direction_pin(&self) -> Result<OutputPin, String> {
        Gpio::new()
            .and_then(|gpio| gpio.get(26))
            .map(|pin| Ok(pin.into_output()))
            .unwrap_or_else(|_| Err("Failed to get direction pin".to_string()))
    }
    pub fn get_step_pin(&self) -> Result<OutputPin, String> {
        Gpio::new()
            .and_then(|gpio| gpio.get(19))
            .map(|pin| Ok(pin.into_output()))
            .unwrap_or_else(|_| Err("Failed to get step pin".to_string()))
    }
    pub fn get_sleep_pin(&self) -> Result<OutputPin, String> {
        Gpio::new()
            .and_then(|gpio| gpio.get(13))
            .map(|pin| Ok(pin.into_output()))
            .unwrap_or_else(|_| Err("Failed to get sleep pin".to_string()))
    }
    pub fn get_reset_pin(&self) -> Result<OutputPin, String> {
        Gpio::new()
            .and_then(|gpio| gpio.get(6))
            .map(|pin| Ok(pin.into_output()))
            .unwrap_or_else(|_| Err("Failed to get reset pin".to_string()))
    }

    pub fn get_enable_pin(&self) -> Result<OutputPin, String> {
        Gpio::new()
            .and_then(|gpio| gpio.get(17))
            .map(|pin| Ok(pin.into_output()))
            .unwrap_or_else(|_| Err("Failed to get enable pin".to_string()))
    }

}
