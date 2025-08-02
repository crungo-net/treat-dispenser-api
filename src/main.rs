use tracing::error;
use treat_dispenser_api::load_app_config;
use treat_dispenser_api::{build_app, configure_logging, start_server, start_power_monitoring_thread};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    configure_logging();

    let config = load_app_config();
    let (app_state, router) = build_app(config.clone());

    if std::env::var_os("DISPENSER_API_TOKEN").map_or(true, |v| v.is_empty()) {
        error!("DISPENSER_API_TOKEN environment variable is not set or is empty");
        std::process::exit(1);
    }

    start_power_monitoring_thread(app_state).await;
    start_server(router, config).await;
}
