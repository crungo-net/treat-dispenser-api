use std::time::Duration;
use rppal::gpio::{Gpio, Level::High, Level::Low};
use tracing::{info, debug};
use tracing_subscriber::field::debug;
use crate::state::{self, DispenserState, DispenserStatus};
use std::sync::{Arc};
use tokio::sync::Mutex;

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
    fn run_motor(&self, steps: u32, direction: Direction, step_mode: &StepMode, hw_state: &Arc<Mutex<DispenserState>>) -> Result<(u32), String>;

    /// Runs the motor for a specified number of degrees in a given direction and step mode.
    /// The number of steps is calculated based on the step mode and the degrees.
    /// Returns the last step index reached after running the motor.
    fn run_motor_degrees(&self, degrees: f32, direction: Direction, step_mode: &StepMode, hw_state: &Arc<Mutex<DispenserState>>) -> Result<(u32), String> {
        let step_count = (degrees / 360.0 * self.get_step_count_for_full_rotation(step_mode) as f32) as u32;
        self.run_motor(step_count, direction, step_mode, hw_state)
    }

    fn get_step_count_for_full_rotation(&self, step_mode: &StepMode) -> u32;
}

pub struct Stepper28BYJ48 {}

impl StepperMotor for Stepper28BYJ48 {
    // todo: handle direction
    fn run_motor(&self, step_count: u32, _direction: Direction, step_mode: &StepMode, _state: &Arc<Mutex<DispenserState>>) -> Result<(u32), String> {
        let delay_between_steps_ms: u64;
        let step_sequence: Vec<[u8; 4]> = match step_mode {

            // 4096 steps for a full rotation in half step mod
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
            },
            // 2048 steps for a full rotation in full step mode
            // 2048/4 = 512 cycles needed for full rotation
            // more torque than half step mode due to two coils being energized at once
            // but needs more time in between steps to avoid overheating
            StepMode::Full => {
                info!("Using full step mode");
                delay_between_steps_ms = 2; 
                vec![
                    [1, 1, 0, 0],
                    [0, 1, 1, 0],
                    [0, 0, 1, 1],
                    [1, 0, 0, 1],
                ]
            },

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
    
                // todo: for the 28BYJ-48/ULN2003, we need to keep track of the sequence index it stops at to avoid jolting the motor
                // when it next runs, particularly if it stops in the middle of the sequence
                // This is not needed for the NEMA-14/A4988, as the driver will handle the sequence automatically
                // For now, we just repeat the sequence from the start
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
            },
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