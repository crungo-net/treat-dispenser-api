use crate::motor::{Direction, StepMode, StepperMotor};

use rppal::gpio::{Gpio};
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

            Ok(gpio) => {
                let mut step_pin = gpio
                    .get(19)
                    .map_err(|e| format!("Failed to get GPIO pin: {}", e))?
                    .into_output();

                let mut dir_pin = gpio
                    .get(26)
                    .map_err(|e| format!("Failed to get GPIO pin: {}", e))?
                    .into_output();

                let mut sleep_pin = gpio
                    .get(13)
                    .map_err(|e| format!("Failed to get GPIO pin: {}", e))?
                    .into_output();

                let mut reset_pin = gpio
                    .get(6)
                    .map_err(|e| format!("Failed to get GPIO pin: {}", e))?
                    .into_output();
                
                sleep_pin.write(rppal::gpio::Level::High);
                reset_pin.write(rppal::gpio::Level::High);


                let _delay_between_steps_ms = 1;

                match direction {
                    Direction::Clockwise => dir_pin.write(rppal::gpio::Level::Low),
                    Direction::CounterClockwise => dir_pin.write(rppal::gpio::Level::High),
                }

                for _ in 0..steps {
                    step_pin.write(rppal::gpio::Level::High);
                    std::thread::sleep(Duration::from_micros(5000));
                    step_pin.write(rppal::gpio::Level::Low);
                    std::thread::sleep(Duration::from_micros(5000));
                }

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
}
