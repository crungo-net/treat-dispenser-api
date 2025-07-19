use crate::motor::{Direction, StepMode, StepperMotor};

use rppal::gpio::{Gpio, OutputPin};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::info;

pub struct StepperNema14 {
    config: Nema14Config,
}

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
                let mut sleep_pin = self.get_sleep_pin()?;
                let mut reset_pin = self.get_reset_pin()?;

                sleep_pin.write(rppal::gpio::Level::High);
                reset_pin.write(rppal::gpio::Level::High);
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
    pub fn new(config: Nema14Config) -> Self {
        StepperNema14 { config }
    }

    pub fn get_direction_pin(&self) -> Result<OutputPin, String> {
        let pin_num = self.config.dir_pin;
        Gpio::new()
            .and_then(|gpio| gpio.get(pin_num))
            .map(|pin| Ok(pin.into_output()))
            .unwrap_or_else(|_| Err("Failed to get direction pin".to_string()))
    }
    pub fn get_step_pin(&self) -> Result<OutputPin, String> {
        let pin_num = self.config.step_pin;
        Gpio::new()
            .and_then(|gpio| gpio.get(pin_num))
            .map(|pin| Ok(pin.into_output()))
            .unwrap_or_else(|_| Err("Failed to get step pin".to_string()))
    }
    pub fn get_sleep_pin(&self) -> Result<OutputPin, String> {
        let pin_num = self.config.sleep_pin;
        Gpio::new()
            .and_then(|gpio| gpio.get(pin_num))
            .map(|pin| Ok(pin.into_output()))
            .unwrap_or_else(|_| Err("Failed to get sleep pin".to_string()))
    }
    pub fn get_reset_pin(&self) -> Result<OutputPin, String> {
        let pin_num = self.config.reset_pin;
        Gpio::new()
            .and_then(|gpio| gpio.get(pin_num))
            .map(|pin| Ok(pin.into_output()))
            .unwrap_or_else(|_| Err("Failed to get reset pin".to_string()))
    }

    pub fn get_enable_pin(&self) -> Result<OutputPin, String> {
        let pin_num = self.config.enable_pin;
        Gpio::new()
            .and_then(|gpio| gpio.get(pin_num))
            .map(|pin| Ok(pin.into_output()))
            .unwrap_or_else(|_| Err("Failed to get enable pin".to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nema14Config {
    pub dir_pin: u8,
    pub step_pin: u8,
    pub sleep_pin: u8,
    pub reset_pin: u8,
    pub enable_pin: u8,
}
