use axum::{
    Json,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::time::Duration;
use tokio::time::timeout;

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Serialize)]
pub struct TimeoutErrorResponse {
    pub error: String,
    pub response_message: String,
}

// ============================================================================
// Timeout Middleware
// ============================================================================

pub async fn timeout_middleware(
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<TimeoutErrorResponse>)> {
    // println!("Request timeout middleware: monitoring request...");

    // Set timeout duration to 1 minute (60 seconds)
    let timeout_duration = Duration::from_secs(60);

    // Wrap the request handling in a timeout
    match timeout(timeout_duration, next.run(req)).await {
        Ok(response) => {
            // println!("Request completed within timeout");
            Ok(response)
        }
        Err(_) => {
            println!("Request timed out after 60 seconds");
            Err((
                StatusCode::REQUEST_TIMEOUT,
                Json(TimeoutErrorResponse {
                    error: "Request Timeout".to_string(),
                    response_message: "Request exceeded the maximum allowed time of 60 seconds"
                        .to_string(),
                }),
            ))
        }
    }
}
