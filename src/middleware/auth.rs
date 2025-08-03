use axum::{
    http::{self, HeaderValue},
    response::Response,
    middleware::{Next},
    extract::Request,
};
use futures::future::BoxFuture;
use crate::error::ApiError;
use tracing::{error, debug};


/// Returns a middleware function that checks for the presence of a valid API token in the request headers.
/// If the token is valid, it allows the request to proceed; otherwise, it returns an `Unauthorized` error.
pub fn create_auth_middleware() -> impl Fn(Request, Next) -> BoxFuture<'static, Result<Response, ApiError>> + Clone {
    move |request: Request, next: Next| {
        Box::pin(async move {
            // Extract the token from the request headers
            let auth_header: Option<&HeaderValue> = request.headers().get(http::header::AUTHORIZATION);
            let token_from_env_result = std::env::var("DISPENSER_API_TOKEN");

            let token = match token_from_env_result {
                Ok(token) => {
                    // Check if the token is set in the environment
                    if token.is_empty() {
                        error!("DISPENSER_API_TOKEN is empty.");
                        return Err(ApiError::Internal("DISPENSER_API_TOKEN is not set".to_string()));
                    }
                    token
                },
                Err(e) => {
                    error!("Failed to read DISPENSER_API_TOKEN from environment: {}", e);
                    return Err(ApiError::Internal("DISPENSER_API_TOKEN could not be read from environment".to_string()));
                }
            };

            // Check if the token matches
            if let Some(auth_header) = auth_header {
                if auth_header == format!("Bearer {}", token).as_str() {
                    return Ok(next.run(request).await);
                }
            }
            error!("Unauthorized request: Invalid or missing token");
            debug!("Expected token: {}", token);
            Err(ApiError::Unauthorized)
        })
    }
}   
