mod dispenser;
mod auth;

use axum::{Router, response::IntoResponse, routing::get};
use env_logger::Env;
use log::{info, error};
use auth::Auth;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .target(env_logger::Target::Stdout)
        .init();

    let _ = std::env::var("DISPENSER_API_TOKEN")
        .expect("Environment variable DISPENSER_API_TOKEN must be set");

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/dispense", get(dispense_treat));

    let port = std::env::var("DISPENSER_API_PORT")
        .unwrap_or_else(|_| "3500".to_string());

    let bind_address = format!("0.0.0.0:{}", port);

    let listener = tokio::net::TcpListener::bind(&bind_address).await.unwrap();
    info!("Listening on {}", bind_address.to_string());

    let shutdown_signal = async {
        // Wait for a shutdown signal (e.g., Ctrl+C)
        tokio::signal::ctrl_c().await.expect("Failed to install Ctrl+C handler");
        info!("Received shutdown signal, shutting down gracefully...");
    };

    tokio::select! {
        _ = init_server(listener, app) => {},
        _ = shutdown_signal => {}
    }

}

async fn init_server(listener: tokio::net::TcpListener, app: Router) {
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> impl IntoResponse {
    info!("Got request to root endpoint");
    "Treat dispenser is online! Binky time!"
}


async fn dispense_treat(_auth: Auth) -> impl IntoResponse {
    match dispenser::dispense() {
        Ok(_) => "Treat dispensed successfully!".to_string(),
        Err(e) => {
            let msg = format!("Failed to dispense treat: {}", e);
            error!("{}", msg);
            msg
        }
    }
}

async fn health_check() -> impl IntoResponse {
    "OK"
}