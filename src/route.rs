use tracing::{info};
use crate::dispenser;
use crate::auth::Auth;
use crate::error::ApiError;
use axum::{response::IntoResponse};

pub async fn root() -> impl IntoResponse {
    "Treat dispenser is online! Binky time!"
}


pub async fn dispense_treat(_auth: Auth) -> Result<&'static str, ApiError> {
    dispenser::dispense()?;
    Ok("Treat dispensed!")
}

pub async fn health_check() -> impl IntoResponse {
    "OK"
}