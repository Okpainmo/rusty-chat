use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Debug, Deserialize)]
pub struct RemoveAdminPayload {
    pub user_id: i64,
}

#[derive(Debug, Serialize)]
pub struct Response {
    response_message: String,
    error: Option<String>,
}

pub async fn remove_room_admin(
    State(state): State<AppState>,
    Path(room_id): Path<i64>,
    Json(payload): Json<RemoveAdminPayload>,
) -> impl IntoResponse {
    let result = sqlx::query(
        r#"
        UPDATE room_members 
        SET role = 'member' 
        WHERE room_id = $1 AND user_id = $2
        "#
    )
    .bind(room_id)
    .bind(payload.user_id)
    .execute(&state.db)
    .await;

    match result {
        Ok(query_result) => {
            if query_result.rows_affected() == 0 {
                (
                    StatusCode::NOT_FOUND,
                    Json(Response {
                        response_message: "User not found in this room".into(),
                        error: Some("Member does not exist or room not found".into()),
                    }),
                )
            } else {
                (
                    StatusCode::OK,
                    Json(Response {
                        response_message: "Room admin removed successfully".into(),
                        error: None,
                    }),
                )
            }
        },
        Err(e) => {
            error!("REMOVE ROOM ADMIN REQUEST FAILED");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Response {
                    response_message: "Failed to remove room admin".into(),
                    error: Some(format!("Database error: {}", e)),
                }),
            )
        }
    }
}
