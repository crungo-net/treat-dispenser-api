mod dispenser;

use axum::{Router, response::IntoResponse, routing::get};
use env_logger::Env;
use log::{info, error};

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .target(env_logger::Target::Stdout)
        .init();
    info!("Logger initialized");

    let app = Router::new()
        .route("/", get(root))
        .route("/dispense", get(dispense_treat));

    // run our app with hyper, listening globally on port 3500
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3500").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> impl IntoResponse {
    "Treat dispenser is online!"
}


async fn dispense_treat() -> impl IntoResponse {
    // Logic to dispense a treat
    match dispenser::dispense() {
        Ok(_) => "Treat dispensed successfully!",
        Err(e) => {
            error!("Failed to dispense treat: {}", e);
            "Failed to dispense treat."
        }
    }
}