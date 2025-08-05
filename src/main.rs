use treat_dispenser_api::load_app_config;
use treat_dispenser_api::{
    build_app, configure_logging, start_power_monitoring_thread, start_server,
};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    configure_logging();

    let config = load_app_config();
    let (app_state, router) = build_app(config.clone());

    start_power_monitoring_thread(app_state).await;
    start_server(router, config).await;
}
