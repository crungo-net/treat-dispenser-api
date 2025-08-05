use crate::application_state::ApplicationState;
use crate::motor::{AsyncStepperMotor, Direction, StepMode, StepperMotor};
use rppal::gpio::{Gpio, Level::Low};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tracing::info;

pub struct Stepper28BYJ48 {}

#[async_trait::async_trait]
impl AsyncStepperMotor for Stepper28BYJ48 {
    async fn run_motor_degrees_async(
        &self,
        degrees: f32,
        direction: &Direction,
        step_mode: &StepMode,
        app_state: &Arc<Mutex<ApplicationState>>,
        _cancel_token: &CancellationToken,
    ) -> Result<u32, String> {
        self.run_motor_degrees(degrees, direction, step_mode, app_state)
    }
}

impl StepperMotor for Stepper28BYJ48 {
    fn get_name(&self) -> String {
        "Stepper28BYJ48".to_string()
    }

    fn run_motor(
        &self,
        step_count: u32,
        direction: &Direction,
        step_mode: &StepMode,
        _app_state: &Arc<Mutex<ApplicationState>>,
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
                // Use the init_stepper_pins function and handle its result properly
                let pins =
                    crate::utils::gpio::init_stepper_pins(&gpio).map_err(|e| format!("{}", e))?;

                let [mut pin1, mut pin2, mut pin3, mut pin4] = pins
                    .try_into()
                    .map_err(|_| format!("Failed to initialize stepper pins."))?;
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Stepper28BYJ48 {
    pub fn new() -> Self {
        Stepper28BYJ48 {}
    }
}
