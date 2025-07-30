use reqwest::Client;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use treat_dispenser_api::build_app;
use treat_dispenser_api::services::status::HealthStatus;

async fn setup() -> (SocketAddr, Client) {
    dotenv::from_filename(".env.test").ok();
    let addr = start_server().await;
    wait_for_server(100).await;
    (addr, Client::new())
}

async fn wait_for_server(millis: u64) {
    tokio::time::sleep(tokio::time::Duration::from_millis(millis)).await;
}

async fn start_server() -> SocketAddr {
    let config_str = r#"
    api:
      listen_address: "127.0.0.1:0"
    motor_cooldown_ms: 5000
    "#;

    let config = treat_dispenser_api::load_app_config_from_str(config_str);
    let app = build_app(config.clone());
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

    addr
}

async fn get_with_auth(
    client: &Client,
    addr: SocketAddr,
    path: &str,
    token: Option<&str>,
) -> reqwest::Response {
    let url = format!("http://{}{}", addr, path);
    let req = client.get(&url);
    let req = if let Some(token) = token {
        req.header("Authorization", format!("Bearer {}", token))
    } else {
        req
    };
    req.send().await.unwrap()
}

async fn get_hardware_status(client: &Client, addr: SocketAddr) -> HealthStatus {
    let response = get_with_auth(client, addr, "/status", None).await;
    assert!(
        response.status().is_success(),
        "Expected success, got: {}",
        response.status()
    );
    response.json::<HealthStatus>().await.unwrap()
}

#[tokio::test]
async fn test_root_endpoint() {
    let (addr, client) = setup().await;
    let response = get_with_auth(&client, addr, "/", None).await;
    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_status_endpoint() {
    let (addr, client) = setup().await;
    wait_for_server(5000).await; // Wait for server to be ready

    let response = get_with_auth(&client, addr, "/status", None).await;
    assert!(response.status().is_success());

    let status_json = response.json::<HealthStatus>().await.unwrap();

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
}

#[tokio::test]
async fn test_dispense_endpoint_unauthorized() {
    let (addr, client) = setup().await;
    let response = get_with_auth(&client, addr, "/dispense", None).await;
    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_dispense_endpoint_authorized() {
    let (addr, client) = setup().await;
    let token = std::env::var("DISPENSER_API_TOKEN").unwrap_or_else(|_| "supersecret".to_string());
    let response = get_with_auth(&client, addr, "/dispense", Some(&token)).await;

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
async fn test_dispense_endpoint_busy_response() {
    let (addr, client) = setup().await;
    let token = std::env::var("DISPENSER_API_TOKEN").unwrap_or_else(|_| "supersecret".to_string());
    let _ = get_with_auth(&client, addr, "/dispense", Some(&token)).await;
    let response = get_with_auth(&client, addr, "/dispense", Some(&token)).await;

    let hardware_status = get_hardware_status(&client, addr).await;

    assert_eq!(response.status(), reqwest::StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        hardware_status.dispenser_status, "Dispensing",
        "Dispenser should be in 'Dispensing' state"
    );

    wait_for_server(3500).await; // wait for mock dispensing to finish
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
