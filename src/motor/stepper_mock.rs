use crate::application_state::ApplicationState;
use crate::motor::{AsyncStepperMotor, Direction, StepMode, StepperMotor};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
pub struct StepperMock {}

impl StepperMock {
    pub fn new() -> Self {
        StepperMock {}
    }
}

#[async_trait::async_trait]
impl AsyncStepperMotor for StepperMock {
    async fn run_motor_degrees_async(
        &self,
        _degrees: f32,
        _direction: &Direction,
        _step_mode: &StepMode,
        _app_state: &Arc<Mutex<ApplicationState>>,
        cancel_token: &CancellationToken,
    ) -> Result<u32, String> {
        // Simulate motor operation
        for _ in 0..5000 {
            if cancel_token.is_cancelled() {
                return Err("Motor operation cancelled".to_string());
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
        Ok(0) // Mock implementation
    }
}

impl StepperMotor for StepperMock {
    fn run_motor(
        &self,
        _steps: u32,
        _direction: &Direction,
        _step_mode: &StepMode,
        _app_state: &Arc<Mutex<ApplicationState>>,
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
