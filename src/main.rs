mod dispenser;
mod auth;
mod error;
mod route;

use axum::{Router, routing::get};
use env_logger::Env;
use log::{info, error};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .target(env_logger::Target::Stdout)
        .init();

    if std::env::var_os("DISPENSER_API_TOKEN").map_or(true, |v| v.is_empty()) {
        error!("DISPENSER_API_TOKEN environment variable is not set or is empty");
        std::process::exit(1);
    } else {
        info!("DISPENSER_API_TOKEN is set");
    }

    let app = Router::new()
        .route("/", get(route::root))
        .route("/health", get(route::health_check))
        .route("/dispense", get(route::dispense_treat));

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
