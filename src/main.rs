use treat_dispenser_api::config::load_app_config;
use treat_dispenser_api::{
    build_app, configure_logging, services::power_monitor, services::weight_monitor, start_server,
};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    configure_logging();

    let config = load_app_config();
    let (app_state, router) = build_app(config.clone());

    power_monitor::start_power_monitoring_thread(&app_state).await;
    weight_monitor::start_weight_monitoring_thread(&app_state).await;
    start_server(router, config).await;
}
