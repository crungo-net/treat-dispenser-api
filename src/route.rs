use log::{info};
use crate::dispenser;
use crate::auth::Auth;
use crate::error::ApiError;
use axum::{response::IntoResponse};

pub async fn root() -> impl IntoResponse {
    info!("Got request to root endpoint");
    "Treat dispenser is online! Binky time!"
}


pub async fn dispense_treat(_auth: Auth) -> Result<&'static str, ApiError> {
    dispenser::dispense()?;
    Ok("Treat dispensed!")
}

pub async fn health_check() -> impl IntoResponse {
    "OK"
}