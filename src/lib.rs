pub mod error;
pub mod middleware;
pub mod motor;
pub mod response;
pub mod routes;
pub mod services;
pub mod state;
pub mod utils;

use axum::extract::ConnectInfo;
use axum::http::Request;
use axum::{Router, routing::get};
use serde_yaml;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;
use tracing::{Level, debug, info};
use tracing_subscriber::EnvFilter;

pub fn configure_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("RUST_LOG").unwrap_or_else(|_| {
                info!("RUST_LOG not set, using default log level 'info'");
                EnvFilter::new("info") // Default log level if not set
            }),
        )
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_writer(std::io::stdout) // log to stdout for compat with containerized environments
        .init();
}

/// Builds the Axum application with routes and shared state.
/// A TraceLayer is added for logging client request details.
pub fn build_app(app_config: AppConfig) -> axum::Router {
    let app_state = Arc::new(Mutex::new(state::ApplicationState::new(app_config)));

    Router::new()
        .route("/", get(routes::root))
        .route("/status", get(routes::status::detailed_health))
        .route("/dispense", get(routes::dispense::dispense_treat))
        .with_state(app_state)
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                let request_ip_addr = request
                    .extensions()
                    .get::<ConnectInfo<SocketAddr>>()
                    .map(|ConnectInfo(addr)| addr.to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                // '%' is tracing syntax used to format the span name
                tracing::span!(
                    Level::INFO,
                    "request",
                    method = %request.method(),
                    uri = %request.uri(),
                    client_ip = %request_ip_addr,
                )
            }),
        )
}

pub async fn start_server(app: Router, config: AppConfig) {
    let bind_address: SocketAddr = format!("{}", config.api.listen_address).parse().unwrap();
    let listener = tokio::net::TcpListener::bind(bind_address)
        .await
        .expect("Failed to bind to address");

    let shutdown_handler = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        info!("Received shutdown signal, shutting down gracefully...");
    };

    info!("Starting server, API listening on {}", bind_address);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_handler)
    .await
    .expect("Failed to start server");
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct ApiConfig {
    pub listen_address: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct AppConfig {
    pub api: ApiConfig,
    pub nema14: Option<crate::motor::stepper_nema14::Nema14Config>,
    pub motor_cooldown_ms: u64,
}

pub fn load_app_config_from_str(config_str: &str) -> AppConfig {
    serde_yaml::from_str(config_str).expect("Failed to parse app config")
}

pub fn load_app_config() -> AppConfig {
    let app_config_path = utils::filesystem::get_config_path();
    let config_str = std::fs::read_to_string(&app_config_path).expect(&format!(
        "Failed to read app config file at {}",
        app_config_path
    ));

    let app_config: AppConfig = load_app_config_from_str(&config_str);

    // Log the config struct as json
    debug!("Parsed app config: {}", serde_json::to_string(&app_config).unwrap_or_default());
    app_config
}
