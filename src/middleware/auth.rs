use crate::error::ApiError;
use axum::{extract::Request, http, middleware::Next, response::Response};
use jsonwebtoken::{DecodingKey, Validation, decode};
use tracing::{debug, warn};

use crate::services::auth::Claims;

pub async fn token_auth_middleware(request: Request, next: Next) -> Result<Response, ApiError> {
    // Extract token from Authorization header
    let auth_header: Option<String> = request
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|auth_value| {
            if auth_value.starts_with("Bearer ") {
                Some(auth_value[7..].to_string())
            } else {
                None
            }
        });

    let jwt_secret_result = std::env::var("DISPENSER_JWT_SECRET");

    let jwt_secret = match jwt_secret_result {
        Ok(secret) => {
            debug!("Using JWT secret from environment variable");
            secret
        }
        Err(_) => {
            return Err(ApiError::Internal(
                "DISPENSER_JWT_SECRET not set in config".to_string(),
            ));
        }
    };

    if let Some(token) = auth_header {
        // Validate token
        match decode::<Claims>(
            &token,
            &DecodingKey::from_secret(jwt_secret.as_ref()),
            &Validation::default(),
        ) {
            Ok(_) => Ok(next.run(request).await),
            Err(_) => Err(ApiError::Unauthorized),
        }
    } else {
        warn!("Authorization header missing or malformed");
        Err(ApiError::Unauthorized)
    }
}
