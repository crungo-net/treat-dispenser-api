use crate::application_state::ApplicationState;
use core::fmt;
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod stepper_28byj48;
pub mod stepper_mock;
pub mod stepper_nema14;

pub enum StepMode {
    Full,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
}

impl fmt::Display for StepMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StepMode::Full => write!(f, "Full"),
            StepMode::Half => write!(f, "Half"),
            StepMode::Quarter => write!(f, "Quarter"),
            StepMode::Eighth => write!(f, "Eighth"),
            StepMode::Sixteenth => write!(f, "Sixteenth"),
        }
    }
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
        app_state: &Arc<Mutex<ApplicationState>>,
    ) -> Result<u32, String>;

    /// Runs the motor for a specified number of degrees in a given direction and step mode.
    /// The number of steps is calculated based on the step mode and the degrees.
    /// Returns the last step index reached after running the motor.
    fn run_motor_degrees(
        &self,
        degrees: f32,
        direction: &Direction,
        step_mode: &StepMode,
        app_state: &Arc<Mutex<ApplicationState>>,
    ) -> Result<u32, String> {
        let step_count =
            (degrees / 360.0 * self.get_step_count_for_full_rotation(step_mode) as f32) as u32;
        self.run_motor(step_count, direction, step_mode, app_state)
    }

    fn get_step_count_for_full_rotation(&self, step_mode: &StepMode) -> u32;

    fn get_name(&self) -> String;

    fn requires_gpio(&self) -> bool {
        true
    }
}
