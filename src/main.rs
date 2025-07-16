use axum::serve;
use std::net::SocketAddr;
use tracing::{error, info};
use treat_dispenser_api::{build_app, configure_logging};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    configure_logging();

    // read the version from Cargo.toml at compile time, bakes this into the binary
    let version = env!("CARGO_PKG_VERSION");
    info!("Starting Treat Dispenser API version {}", version);

    if std::env::var_os("DISPENSER_API_TOKEN").map_or(true, |v| v.is_empty()) {
        error!("DISPENSER_API_TOKEN environment variable is not set or is empty");
        std::process::exit(1);
    }

    let app = build_app();
    let port = std::env::var("DISPENSER_API_PORT").unwrap_or_else(|_| "3500".to_string());
    let bind_address: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    let listener = tokio::net::TcpListener::bind(bind_address).await.unwrap();

    info!("Listening on {}", bind_address);

    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        info!("Received shutdown signal, shutting down gracefully...");
    };

    serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal)
    .await
    .unwrap();
}
