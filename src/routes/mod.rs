pub mod auth;
pub mod dispense;
pub mod status;
pub mod sensors;

use axum::response::IntoResponse;

pub async fn root() -> impl IntoResponse {
    "Treat dispenser is online! Binky time!"
}
