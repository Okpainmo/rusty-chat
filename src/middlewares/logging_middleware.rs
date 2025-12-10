use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;
use tracing::info;

// ============================================================================
// Logging Middleware
// ============================================================================

pub async fn logging_middleware(req: Request, next: Next) -> Response {
    let path = req.uri().path().to_string();
    let start_time = Instant::now();
    let start_timestamp = chrono::Local::now();

    // Process the request
    let response = next.run(req).await;

    // Calculate duration and end time
    let duration = start_time.elapsed();
    let end_timestamp = chrono::Local::now();

    info!(
        "Path: {} | Start: {} | End: {} | Duration: {:.3}ms",
        path,
        start_timestamp.format("%H:%M:%S%.3f"),
        end_timestamp.format("%H:%M:%S%.3f"),
        duration.as_secs_f64() * 1000.0,
    );

    response
}
