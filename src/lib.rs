pub mod services;
pub mod error;
pub mod motor;
pub mod response;
pub mod state;
pub mod middleware;
pub mod routes;

use axum::extract::ConnectInfo;
use axum::http::Request;
use axum::{Router, routing::get};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;
use tracing::{Level, info};
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
pub fn build_app() -> axum::Router {
    let hw_state = Arc::new(Mutex::new(state::DispenserState::new()));

    Router::new()
        .route("/", get(routes::root))
        .route("/status", get(routes::status::detailed_health))
        .route("/dispense", get(routes::dispense::dispense_treat))
        .with_state(hw_state)
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
