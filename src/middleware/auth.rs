use crate::error::ApiError;
use axum::{
    extract::Request,
    http::{self, HeaderValue},
    middleware::Next,
    response::Response,
};
use futures::future::BoxFuture;
use jsonwebtoken::{decode, DecodingKey, Validation};
use tracing::error;

use crate::services::auth::Claims;

/// Returns a middleware function that checks for the presence of a valid API token in the request headers.
/// If the token is valid, it allows the request to proceed; otherwise, it returns an `Unauthorized` error.
pub fn create_auth_middleware()
-> impl Fn(Request, Next) -> BoxFuture<'static, Result<Response, ApiError>> + Clone {
    move |request: Request, next: Next| {
        Box::pin(async move {
            // Extract the token from the request headers
            let auth_header: Option<&HeaderValue> =
                request.headers().get(http::header::AUTHORIZATION);
            let token_from_env_result = std::env::var("DISPENSER_API_TOKEN");

            let token = match token_from_env_result {
                Ok(token) => {
                    // Check if the token is set in the environment
                    if token.is_empty() {
                        error!("DISPENSER_API_TOKEN is empty.");
                        return Err(ApiError::Internal(
                            "DISPENSER_API_TOKEN is not set".to_string(),
                        ));
                    }
                    token
                }
                Err(e) => {
                    error!("Failed to read DISPENSER_API_TOKEN from environment: {}", e);
                    return Err(ApiError::Internal(
                        "DISPENSER_API_TOKEN could not be read from environment".to_string(),
                    ));
                }
            };

            // Check if the token matches
            if let Some(auth_header) = auth_header {
                if auth_header == format!("Bearer {}", token).as_str() {
                    return Ok(next.run(request).await);
                }
            }
            Err(ApiError::Unauthorized)
        })
    }
}

pub async fn token_auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Extract token from Authorization header
    let auth_header: Option<String> =
        request.headers().get(http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|auth_value| {
            if auth_value.starts_with("Bearer ") {
                Some(auth_value[7..].to_string())
            } else {
                None
            }
        });
    
    if let Some(token) = auth_header {
        // Validate token
        match decode::<Claims>(
            &token,
            &DecodingKey::from_secret("supersecret".as_ref()),
            &Validation::default(),
        ) {
            Ok(_) => Ok(next.run(request).await),
            Err(_) => Err(ApiError::Unauthorized),
        }
    } else {
        error!("Authorization header missing or malformed");
        Err(ApiError::Unauthorized)
    }
}