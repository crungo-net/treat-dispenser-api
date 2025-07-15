use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::fmt;
use tracing::error;

#[derive(Debug)]
pub enum ApiError {
    Unauthorized,
    Busy(String),
    Hardware(String),
    BadRequest(String),
    Internal(String),
}

// Implementing the Display trait allows us to convert ApiError into a string representation
impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Unauthorized => write!(f, "Unauthorized request"),
            ApiError::Busy(msg) => write!(f, "Dispenser is busy: {}", msg),
            ApiError::Hardware(msg) => write!(f, "Hardware error: {}", msg),
            ApiError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            ApiError::Internal(msg) => write!(f, "Internal server error: {}", msg),
        }
    }
}

// tells axum how to convert ApiError into an HTTP response
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        error!("{}", self);
        let (status, body) = match self {
            ApiError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "Unauthorized request".to_string())
            }
            ApiError::Hardware(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ApiError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ApiError::Busy(_) => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()),
        };
        (status, body).into_response()
    }
}
