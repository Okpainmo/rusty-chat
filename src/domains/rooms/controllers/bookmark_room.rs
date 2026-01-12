use crate::AppState;
use crate::middlewares::auth_sessions_middleware::SessionsMiddlewareOutput;
use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Serialize;
use tracing::error;

#[derive(Debug, Serialize)]
pub struct Response {
    response_message: String,
    error: Option<String>,
}

pub async fn bookmark_room(
    State(state): State<AppState>,
    Extension(session): Extension<SessionsMiddlewareOutput>,
    Path(room_id): Path<i64>,
) -> impl IntoResponse {
    let user_id = session.user.id;

    // Use array_append to add user_id to bookmarked_by if not already present
    // Postgres array_append adds duplicates, so we might want to check first or distinct.
    // A smarter query: UPDATE rooms SET bookmarked_by = array_append(bookmarked_by, $1) WHERE id = $2 AND NOT ($1 = ANY(bookmarked_by))

    let result = sqlx::query(
        r#"
        UPDATE rooms 
        SET bookmarked_by = array_append(bookmarked_by, $1) 
        WHERE id = $2 AND NOT ($1 = ANY(bookmarked_by))
        "#,
    )
    .bind(user_id)
    .bind(room_id)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => (
            StatusCode::OK,
            Json(Response {
                response_message: "Room bookmarked successfully".into(),
                error: None,
            }),
        ),
        Err(e) => {
            error!("BOOKMARK ROOM REQUEST FAILED!");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Response {
                    response_message: "Failed to bookmark room".into(),
                    error: Some(format!("Database error: {}", e)),
                }),
            )
        }
    }
}
