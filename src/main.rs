use tracing::{error, info};
use treat_dispenser_api::{build_app, start_server, configure_logging};
use treat_dispenser_api::load_app_config;

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
    let config = load_app_config();
    let app = build_app(config.clone());
    start_server(app, config).await;
}
