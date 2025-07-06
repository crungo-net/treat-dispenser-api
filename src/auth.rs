use axum::{
    extract::{FromRequestParts},
    http::{request::Parts, StatusCode}
};
use log::{debug, info};
use crate::error::ApiError;

pub struct Auth;

impl<S> FromRequestParts<S> for Auth 
where 
    // trait bound on S, Send: type can be transferred across threads, Sync: type can be referenced from multiple threads
    // S must implement Send and Sync traits
    // Without this, compiler will not allow us to use this extractor in a multi-threaded context
    S: Send + Sync, 
{
    // Error type that will be returned if the request does not meet the requirements
    // In this case, we use StatusCode to indicate an error
    //type Rejection = (StatusCode, &'static str);
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Here you would typically check for an authorization header or token
        let expected_token = std::env::var("DISPENSER_API_TOKEN").unwrap();
        let auth_header = parts.headers.get("Authorization").and_then(|header| header.to_str().ok()).unwrap_or("NONE");

        // Validate the auth header
        if auth_header == "Bearer " .to_string() + &expected_token {
            // If the token matches, return Ok with Auth instance
            // This indicates that the request is authorized 
            return Ok(Auth);
        }
        debug!("Authorization failed: expected token {}, got {}", expected_token, auth_header);
        Err(ApiError::Unauthorized)
    }
}