use crate::motor::{Direction, StepMode, StepperMotor};
use std::time::Duration;
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
