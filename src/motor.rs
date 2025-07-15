use rppal::gpio::{Gpio, Level::High, Level::Low};
use std::time::Duration;
use tracing::{debug, info};

pub enum StepMode {
    Full,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
}

pub enum Direction {
    Clockwise,
    CounterClockwise,
}

pub trait StepperMotor {
    fn run_motor(
        &self,
        steps: u32,
        direction: &Direction,
        step_mode: &StepMode,
    ) -> Result<u32, String>;

    /// Runs the motor for a specified number of degrees in a given direction and step mode.
    /// The number of steps is calculated based on the step mode and the degrees.
    /// Returns the last step index reached after running the motor.
    fn run_motor_degrees(
        &self,
        degrees: f32,
        direction: &Direction,
        step_mode: &StepMode,
    ) -> Result<u32, String> {
        let step_count =
            (degrees / 360.0 * self.get_step_count_for_full_rotation(step_mode) as f32) as u32;
        self.run_motor(step_count, direction, step_mode)
    }

    fn get_step_count_for_full_rotation(&self, step_mode: &StepMode) -> u32;

    fn get_name(&self) -> String;

    fn requires_gpio(&self) -> bool {
        true
    }

}

pub struct Stepper28BYJ48 {}

impl StepperMotor for Stepper28BYJ48 {
    fn get_name(&self) -> String {
        "Stepper28BYJ48".to_string()
    }

    fn run_motor(
        &self,
        step_count: u32,
        direction: &Direction,
        step_mode: &StepMode,
    ) -> Result<u32, String> {
        let delay_between_steps_ms: u64;
        let mut step_sequence: Vec<[u8; 4]> = match step_mode {
            StepMode::Half => {
                info!("Using half step mode");
                delay_between_steps_ms = 1;
                vec![
                    [1, 0, 0, 0],
                    [1, 1, 0, 0],
                    [0, 1, 0, 0],
                    [0, 1, 1, 0],
                    [0, 0, 1, 0],
                    [0, 0, 1, 1],
                    [0, 0, 0, 1],
                    [1, 0, 0, 1],
                ]
            }
            // more torque than half step mode due to two coils being energized at once
            // but needs more time in between steps to avoid overheating
            StepMode::Full => {
                info!("Using full step mode");
                delay_between_steps_ms = 2;
                vec![[1, 1, 0, 0], [0, 1, 1, 0], [0, 0, 1, 1], [1, 0, 0, 1]]
            }

            _ => {
                return Err("Unsupported step mode".to_string());
            }
        };
        match Gpio::new() {
            Ok(gpio) => {
                let mut pin1 = self.get_pin(&gpio, 26)?;
                let mut pin2 = self.get_pin(&gpio, 19)?;
                let mut pin3 = self.get_pin(&gpio, 13)?;
                let mut pin4 = self.get_pin(&gpio, 6)?;

                info!("Starting motor with {} steps", step_count);

                let mut last_step_index: u32 = 0;

                match direction {
                    Direction::Clockwise => {
                        info!("Running motor in clockwise direction");
                    }
                    Direction::CounterClockwise => {
                        info!("Running motor in counter-clockwise direction");
                        step_sequence.reverse();
                    }
                }

                for step in 0..step_count {
                    let index = step % step_sequence.len() as u32;
                    last_step_index = index;

                    let sequence = &step_sequence[index as usize];
                    pin1.write(sequence[0].into());
                    pin2.write(sequence[1].into());
                    pin3.write(sequence[2].into());
                    pin4.write(sequence[3].into());
                    std::thread::sleep(Duration::from_millis(delay_between_steps_ms));
                }

                pin1.write(Low);
                pin2.write(Low);
                pin3.write(Low);
                pin4.write(Low);
                info!("Motor operation completed");

                Ok(last_step_index)
            }
            Err(e) => Err(format!("Failed to create local Gpio instance: {}", e)),
        }
    }

    fn get_step_count_for_full_rotation(&self, step_mode: &StepMode) -> u32 {
        match step_mode {
            StepMode::Full => 2048,
            StepMode::Half => 4096,
            StepMode::Quarter => 8192,
            StepMode::Eighth => 16384,
            StepMode::Sixteenth => 32768,
        }
    }
}

impl Stepper28BYJ48 {
    pub fn new() -> Self {
        Stepper28BYJ48 {}
    }

    pub fn get_pin(&self, gpio: &Gpio, pin_number: u8) -> Result<rppal::gpio::OutputPin, String> {
        match gpio.get(pin_number) {
            Ok(pin) => Ok(pin.into_output()),
            Err(e) => Err(format!("Failed to get GPIO pin {}: {}", pin_number, e)),
        }
    }
}

pub struct StepperNema14 {}

pub struct StepperMock {}

impl StepperMock {
    pub fn new() -> Self {
        StepperMock {}
    }
}

impl StepperMotor for StepperMock {
    fn run_motor(
        &self,
        _steps: u32,
        _direction: &Direction,
        _step_mode: &StepMode,
    ) -> Result<u32, String> {
        std::thread::sleep(Duration::from_millis(3000)); // Simulate motor operation
        Ok(0) // Mock implementation
    }

    fn get_step_count_for_full_rotation(&self, _step_mode: &StepMode) -> u32 {
        2048 // Mock implementation
    }

    fn get_name(&self) -> String {
        "StepperMock".to_string()
    }

    fn requires_gpio(&self) -> bool {
        false
    }
}