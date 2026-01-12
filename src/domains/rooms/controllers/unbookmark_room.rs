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

pub async fn unbookmark_room(
    State(state): State<AppState>,
    Extension(session): Extension<SessionsMiddlewareOutput>,
    Path(room_id): Path<i64>,
) -> impl IntoResponse {
    let user_id = session.user.id;

    let result = sqlx::query(
        r#"
        UPDATE rooms 
        SET bookmarked_by = array_remove(bookmarked_by, $1) 
        WHERE id = $2
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
                response_message: "Room unbookmarked successfully".into(),
                error: None,
            }),
        ),
        Err(e) => {
            error!("UNBOOKMARK ROOM REQUEST FAILED!");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Response {
                    response_message: "Failed to unbookmark room".into(),
                    error: Some(format!("Database error: {}", e)),
                }),
            )
        }
    }
}
