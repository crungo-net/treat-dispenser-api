use reqwest::Client;
use treat_dispenser_api::services::weight_monitor::start_weight_monitoring_thread;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Once;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tracing::info;
use treat_dispenser_api::application_state::ApplicationState;
use treat_dispenser_api::build_app;
use treat_dispenser_api::services::power_monitor::start_power_monitoring_thread;
use treat_dispenser_api::services::status::StatusResponse;

async fn setup(config: Option<Box<&str>>) -> (SocketAddr, Client, Arc<Mutex<ApplicationState>>) {
    dotenv::from_filename(".env.test").ok();
    init_logging();
    let (addr, app_state) = start_server(config).await;
    wait_for_server(100).await;
    (addr, Client::new(), app_state)
}

static INIT: Once = Once::new();

pub fn init_logging() {
    INIT.call_once(|| {
        // Initialize logging
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_ansi(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .init();
    });
}

async fn wait_for_server(millis: u64) {
    tokio::time::sleep(tokio::time::Duration::from_millis(millis)).await;
}

async fn start_server(config: Option<Box<&str>>) -> (SocketAddr, Arc<Mutex<ApplicationState>>) {
    let config_str = config.unwrap_or_else(|| {
        Box::new(
        r#"
        api:
          listen_address: "127.0.0.1:0"
          admin_user: "admin"
          admin_password: "password"
        power_monitor:
          sensor: "SensorMock"
          motor_current_limit_amps: 0.7
        weight_monitor:
          sensor: "SensorMock"
        motor:
          motor_type: "StepperMock"
          cooldown_ms: 5000
        "#,
        )
    });
    info!("Using config: {}", config_str);

    let config = treat_dispenser_api::config::load_app_config_from_str(config_str.as_ref());
    let (_app_state, app) = build_app(config.clone());
    let listener = TcpListener::bind(config.api.listen_address).await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    (addr, _app_state)
}

async fn login(
    client: &Client,
    addr: SocketAddr,
    username: &str,
    password: &str,
) -> treat_dispenser_api::services::auth::LoginResponse {
    let url = format!("http://{}/login", addr);
    let req = client.post(&url);
    let req = req.json(&serde_json::json!({
        "username": username,
        "password": password
    }));
    let response = req.send().await.unwrap();
    // deserialize the response to get the token (LoginResponse)
    response
        .json::<treat_dispenser_api::services::auth::LoginResponse>()
        .await
        .unwrap()
}

async fn get_with_auth(client: &Client, addr: SocketAddr, path: &str) -> reqwest::Response {
    let login_response = login(client, addr, "admin", "password").await;
    let token = login_response.token;
    let url = format!("http://{}{}", addr, path);
    let req = client.get(&url);
    let req = req.header("Authorization", format!("Bearer {}", token));
    req.send().await.unwrap()
}

async fn post_with_auth(client: &Client, addr: SocketAddr, path: &str) -> reqwest::Response {
    let login_response = login(client, addr, "admin", "password").await;
    let token = login_response.token;
    let url = format!("http://{}{}", addr, path);
    let req = client.post(&url);
    let req = req.header("Authorization", format!("Bearer {}", token));
    req.send().await.unwrap()
}

async fn get_hardware_status(client: &Client, addr: SocketAddr) -> StatusResponse {
    let response = get_with_auth(client, addr, "/status").await;
    assert!(
        response.status().is_success(),
        "Expected success, got: {}",
        response.status()
    );
    response.json::<StatusResponse>().await.unwrap()
}

#[tokio::test]
async fn test_root_endpoint() {
    let (addr, client, _) = setup(None).await;
    let response = get_with_auth(&client, addr, "/").await;
    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_status_endpoint() {
    let (addr, client, _) = setup(None).await;
    wait_for_server(5000).await; // Wait for server to be ready

    let response = get_with_auth(&client, addr, "/status").await;
    assert!(response.status().is_success());

    let status_json = response.json::<StatusResponse>().await.unwrap();

    assert!(status_json.gpio_available == false);
    assert!(status_json.treats_available == false);
    assert!(
        status_json.uptime_seconds > 0,
        "Uptime should be greater than 0"
    );
    assert!(status_json.dispenser_status == "Operational");
    assert!(status_json.last_dispensed.is_none());
    assert!(status_json.last_error_msg.is_none());
    assert!(status_json.last_error_time.is_none());
    assert_eq!(status_json.version, env!("CARGO_PKG_VERSION"));
    assert_eq!(status_json.motor, "StepperMock");

    // since no real power sensor is connected, and the power monitoring thread is not started in this test
    // the power readings should be the default values
    assert_eq!(status_json.motor_voltage_volts, Some(0.0));
    assert_eq!(status_json.motor_current_amps, Some(0.0));
    assert_eq!(status_json.motor_power_watts, Some(0.0));
    assert_eq!(status_json.motor_power_sensor, "SensorMock");
}

#[tokio::test]
async fn test_power_monitoring_thread() {
    let (addr, client, app_state) = setup(None).await;
    start_power_monitoring_thread(&app_state).await;
    wait_for_server(5000).await; // Wait for server to be ready

    let response = get_with_auth(&client, addr, "/status").await;
    assert!(response.status().is_success());

    let status_json = response.json::<StatusResponse>().await.unwrap();

    assert_eq!(status_json.motor_voltage_volts, Some(12.0));
    assert_eq!(status_json.motor_current_amps, Some(0.6));
    assert_eq!(status_json.motor_power_watts, Some(0.5));
    assert_eq!(status_json.motor_power_sensor, "SensorMock");
}

#[tokio::test]
async fn test_weight_monitoring_thread() {
    let (addr, client, app_state) = setup(None).await;
    start_weight_monitoring_thread(&app_state).await;
    wait_for_server(5000).await; // Wait for server to be ready

    let response = get_with_auth(&client, addr, "/status").await;
    assert!(response.status().is_success());

    let status_json = response.json::<StatusResponse>().await.unwrap();

    assert_eq!(status_json.remaining_treats_grams, 12345.0);
}

#[tokio::test]
async fn test_dispense_endpoint_unauthorized() {
    let (addr, client, _) = setup(None).await;
    let req = client.post(format!("http://{}/dispense", addr));
    let response = req.send().await.unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_dispense_endpoint_authorized() {
    let (addr, client, _) = setup(None).await;
    let response = post_with_auth(&client, addr, "/dispense").await;

    let health_status = get_hardware_status(&client, addr).await;

    assert!(
        response.status().is_success(),
        "Expected success, got: {}",
        response.status()
    );
    assert!(
        health_status.dispenser_status == "Dispensing",
        "Dispenser should be in 'Dispensing' state"
    );
}

#[tokio::test]
async fn test_dispense_endpoint_overcurrent_protection() {
    let (addr, client, app_state) = setup(Some(Box::new(
        r#"
        api:
          listen_address: "127.0.0.1:0"
          admin_user: "admin"
          admin_password: "password"
        power_monitor:
          sensor: "SensorMock"
          motor_current_limit_amps: 0.1
        weight_monitor:
          sensor: "SensorMock"
        motor:
          motor_type: "StepperMock"
          cooldown_ms: 5000
        "#,
    )))
    .await;
    start_power_monitoring_thread(&app_state).await;

    let response = post_with_auth(&client, addr, "/dispense").await;

    wait_for_server(7500).await; // wait for mock dispensing to finish
    let hardware_status = get_hardware_status(&client, addr).await;

    assert!(
        response.status().is_success(),
        "Expected success, got: {}",
        response.status()
    );
    assert_eq!(
        hardware_status.dispenser_status, "Cancelled",
        "Dispenser should be in 'Cancelled' state"
    );
}

#[tokio::test]
async fn test_dispense_endpoint_busy_response() {
    let (addr, client, _) = setup(None).await;
    let _ = post_with_auth(&client, addr, "/dispense").await;
    let response = post_with_auth(&client, addr, "/dispense").await;

    let hardware_status = get_hardware_status(&client, addr).await;

    assert_eq!(response.status(), reqwest::StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        hardware_status.dispenser_status, "Dispensing",
        "Dispenser should be in 'Dispensing' state"
    );

    wait_for_server(12500).await; // wait for mock dispensing to finish
    let hardware_status = get_hardware_status(&client, addr).await;
    assert_eq!(
        hardware_status.dispenser_status, "Cooldown",
        "Dispenser should be in 'Cooldown' state after dispensing"
    );

    wait_for_server(5500).await; // Wait for cooldown period to finish
    let hardware_status = get_hardware_status(&client, addr).await;
    assert_eq!(
        hardware_status.dispenser_status, "Operational",
        "Dispenser should be back to 'Operational' state after cooldown"
    );
}

#[tokio::test]
async fn test_cancel_dispense_endpoint() {
    let (addr, client, _) = setup(None).await;
    let response = post_with_auth(&client, addr, "/dispense").await;

    assert!(
        response.status().is_success(),
        "Expected success, got: {}",
        response.status()
    );

    // Cancel the dispense operation
    let cancel_response = post_with_auth(&client, addr, "/cancel").await;
    assert!(
        cancel_response.status().is_success(),
        "Expected success, got: {}",
        cancel_response.status()
    );

    // Check the status after cancellation
    let hardware_status = get_hardware_status(&client, addr).await;
    assert_eq!(
        hardware_status.dispenser_status, "Cancelled",
        "Dispenser should be in 'Cancelled' state"
    );
}
