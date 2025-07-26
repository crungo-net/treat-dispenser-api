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
                let mut step_pin = self.get_output_pin(self.config.step_pin)?;
                let mut dir_pin = self.get_output_pin(self.config.dir_pin)?;
                let mut sleep_pin = self.get_output_pin(self.config.sleep_pin)?;
                let mut reset_pin = self.get_output_pin(self.config.reset_pin)?;
                let mut enable_pin = self.get_output_pin(self.config.enable_pin)?;

                sleep_pin.write(rppal::gpio::Level::High);
                reset_pin.write(rppal::gpio::Level::High);
                enable_pin.write(rppal::gpio::Level::Low); // Enable the motor

                match direction {
                    Direction::Clockwise => dir_pin.write(rppal::gpio::Level::High),
                    Direction::CounterClockwise => dir_pin.write(rppal::gpio::Level::Low),
                }

                let step_speed_us = self.config.step_speed_us.or(Some(1000)).unwrap();

                let mut i = 0;
                let mut is_dir_high = match direction {
                    Direction::Clockwise => true,
                    Direction::CounterClockwise => false,
                };
                for _ in 0..steps {
                    // toggle direction pin every 120 steps (200 is full rotation), helps prevent treats from jamming
                    i += 1;
                    if i % 120 == 0 {
                        if is_dir_high {
                            dir_pin.write(rppal::gpio::Level::Low);
                            is_dir_high = false;
                        } else {
                            dir_pin.write(rppal::gpio::Level::High);
                            is_dir_high = true;
                        }
                        info!("Direction pin toggled at step {}", i);
                        i = 0; // Reset the counter after toggling
                    }

                    // pulse the step pin to move motor shaft
                    step_pin.write(rppal::gpio::Level::High);
                    std::thread::sleep(Duration::from_micros(step_speed_us));
                    step_pin.write(rppal::gpio::Level::Low);
                    std::thread::sleep(Duration::from_micros(step_speed_us));
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

    fn get_output_pin(&self, pin_num: u8) -> Result<OutputPin, String> {
        Gpio::new()
            .and_then(|gpio| gpio.get(pin_num))
            .map(|pin| Ok(pin.into_output()))
            .unwrap_or_else(|_| Err(format!("Failed to get pin {}", pin_num)))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nema14Config {
    pub dir_pin: u8,
    pub step_pin: u8,
    pub sleep_pin: u8,
    pub reset_pin: u8,
    pub enable_pin: u8,
    pub step_speed_us: Option<u64>, // Speed in microseconds per step
}
