pub mod auth;
pub mod dispense;
pub mod sensors;
pub mod status;

use axum::response::IntoResponse;

pub async fn root() -> impl IntoResponse {
    "Treat dispenser is online! Binky time!"
}
