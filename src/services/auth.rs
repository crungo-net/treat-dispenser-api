use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::error::ApiError;

#[derive(Serialize, Deserialize, Clone)]
pub struct LoginRequest {
    pub username: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    token: String,
    expires_at: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: u64,
}

/// Validates user credentials and generates a JWT token if successful.
/// The token is valid for one week.
pub async fn handle_login(
    payload: LoginRequest,
) -> Result<LoginResponse, ApiError> {
    // todo: implement user lookup and password validation
    // for now, we use a hardcoded username and password for demonstration purposes.
    if payload.username == "admin" && payload.password == "password" {
        // Create JWT token that expires in one year
        let expiration = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::days(7))
            .expect("invalid timestamp")
            .timestamp() as u64;

        let claims = Claims {
            sub: payload.username,
            exp: expiration,
        };

        // todo: use a secure secret key in production
        let token_result = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret("supersecret".as_ref()),
        );
        let token = match token_result {
            Ok(t) => t,
            Err(e) => {
                error!("Token creation error: {}", e);
                return Err(ApiError::Internal("Token creation failed".to_string()));
            }
        };

        Ok(LoginResponse {
            token,
            expires_at: expiration,
        })
    } else {
        Err(ApiError::Unauthorized)
    }

}
