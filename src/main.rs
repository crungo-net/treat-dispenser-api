mod auth;
mod dispenser;
mod error;
mod response;
mod route;
mod state;

use axum::extract::ConnectInfo;
use axum::http::Request;
use axum::{Router, routing::get, serve};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::trace::{DefaultOnRequest, TraceLayer};
use tracing::{Level, error, info};
use tracing_subscriber::EnvFilter;
use std::thread;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("RUST_LOG").unwrap_or_else(|_| {
                info!("RUST_LOG not set, using default log level 'info'");
                EnvFilter::new("info") // Default log level if not set
            }),
        )
        .with_writer(std::io::stdout) // log to stdout for compat with containerized environments
        .init();

    if std::env::var_os("DISPENSER_API_TOKEN").map_or(true, |v| v.is_empty()) {
        error!("DISPENSER_API_TOKEN environment variable is not set or is empty");
        std::process::exit(1);
    }

    // Initialize hardware state that will be shared across requests.
    // Hardware state must be shared across threads so we use Arc and Mutex to
    // allow safe concurrent access.
    let hw_state = Arc::new(Mutex::new(state::DispenserState::new()));

    let app = Router::new()
        .route("/", get(route::root))
        .route("/health", get(route::health_check))
        .route("/health/detailed", get(route::detailed_health))
        .route("/dispense", get(route::dispense_treat))
        .with_state(hw_state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let addr = request
                        .extensions()
                        .get::<ConnectInfo<SocketAddr>>()
                        .map(|ConnectInfo(addr)| addr.to_string())
                        .unwrap_or_else(|| "unknown".to_string());

                    let thread_id = format!("{:?}", thread::current().id());

                    // display format: "method=GET uri=/dispense client_ip=0.0.0.0 thread=ThreadId"
                    // the % is special syntax for formatting in tracing
                    // uses Display trait to format the values
                    tracing::span!(
                        Level::INFO,
                        "request",
                        method = %request.method(),
                        uri = %request.uri(),
                        client_ip = %addr,
                        thread = %thread_id
                    )
                })
                .on_request(DefaultOnRequest::new().level(Level::INFO)),
        );

    let port = std::env::var("DISPENSER_API_PORT").unwrap_or_else(|_| "3500".to_string());

    let bind_address: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    info!("Listening on {}", bind_address);

    let listener = tokio::net::TcpListener::bind(bind_address).await.unwrap();

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
