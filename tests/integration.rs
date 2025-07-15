use reqwest::Client;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use treat_dispenser_api::build_app;

async fn setup() -> (SocketAddr, Client) {
    dotenv::from_filename(".env.test").ok();
    let addr = start_server().await;
    wait_for_server().await;
    (addr, Client::new())
}

async fn wait_for_server() {
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}

async fn start_server() -> SocketAddr {
    let app = build_app();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
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

#[tokio::test]
async fn test_root_endpoint() {
    let (addr, client) = setup().await;
    let response = get_with_auth(&client, addr, "/", None).await;
    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_status_endpoint() {
    let (addr, client) = setup().await;
    let response = get_with_auth(&client, addr, "/status", None).await;
    assert!(response.status().is_success());
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
    assert!(
        response.status().is_success()
            || response.status() == reqwest::StatusCode::INTERNAL_SERVER_ERROR,
        "Expected success or hardware error (internal server error), got: {}",
        response.status()
    );
}
