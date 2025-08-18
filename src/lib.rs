pub mod application_state;
pub mod error;
pub mod middleware;
pub mod motor;
pub mod routes;
pub mod sensors;
pub mod services;
pub mod utils;
pub mod config;

use axum::extract::ConnectInfo;
use axum::http::{Method, Request, StatusCode};
use axum::{Router, routing::get, routing::post};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::{DefaultOnFailure, TraceLayer};
use tracing::{Level, error, info, trace, warn};
use tracing_subscriber::EnvFilter;

use crate::application_state::ApplicationState;
use crate::config::AppConfig;

pub fn configure_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("RUST_LOG").unwrap_or_else(|_| {
                info!("RUST_LOG not set, using default log level 'info'");
                EnvFilter::new("info") // Default log level if not set
            }),
        )
        .with_thread_ids(true)
        .with_thread_names(false)
        .with_writer(std::io::stdout) // log to stdout for compat with containerized environments
        .init();
}

/// Builds the Axum application with routes and shared state.
/// A TraceLayer is added for logging client request details.
pub fn build_app(app_config: AppConfig) -> (Arc<Mutex<ApplicationState>>, axum::Router) {
    let app_state = Arc::new(Mutex::new(ApplicationState::new(
        app_config,
    )));

    let cors = CorsLayer::new()
        .allow_origin(Any) // Allow all origins for simplicity, adjust as needed
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any);

    let public_routes = Router::new()
        .route("/", get(routes::root))
        .route(
            "/favicon.ico",
            get(|| async { axum::http::StatusCode::NO_CONTENT }),
        ) // avoids 401 and 404 errors for browser requests to the API, which sometimes request favicon.ico
        .route("/login", post(routes::auth::login))
        .route("/status", get(routes::status::detailed_health));

    let protected_routes = Router::new()
        .route("/dispense", post(routes::dispense::dispense_treat))
        .route("/cancel", post(routes::dispense::cancel_dispense))
        .route("/tare", post(routes::sensors::tare_weight_sensor))
        .route("/calibrate", post(routes::sensors::calibrate_weight_sensor))
        .layer(axum::middleware::from_fn(
            middleware::auth::token_auth_middleware,
        ));

    let merged_routes = public_routes.merge(protected_routes);

    return (
        app_state.clone(),
        merged_routes.with_state(app_state).layer(cors).layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
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
                })
                .on_failure(DefaultOnFailure::new().level(tracing::Level::WARN)) // log http failures at WARN level
                .on_request(log_http_request)
                .on_response(log_http_response_code),
        ),
    );
}

/// Starts the Axum server with the provided router and configuration.
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

// This function is called by the TraceLayer to log the response status code
// It overrides the default behaviour such that we can log specific status codes differently
fn log_http_response_code<B>(
    response: &axum::http::Response<B>,
    _latency: Duration,
    _span: &tracing::Span,
) {
    match response.status() {
        StatusCode::UNAUTHORIZED
        | StatusCode::FORBIDDEN
        | StatusCode::NOT_FOUND
        | StatusCode::TOO_MANY_REQUESTS => {
            warn!("response finished: {}", response.status())
        }
        StatusCode::INTERNAL_SERVER_ERROR => {
            error!("response finished: {}", response.status())
        }
        _ => {
            trace!("response finished: {}", response.status())
        }
    }
}

fn log_http_request<B>(request: &Request<B>, _span: &tracing::Span) {
    let request_ip_addr = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(addr)| addr.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    //span.record("client_ip", &request_ip_addr);
    //span.record("method", &request.method().to_string());
    //span.record("uri", &request.uri().to_string());

    trace!(
        "Received request: {}, {} {}",
        request_ip_addr,
        request.method(),
        request.uri()
    );
}