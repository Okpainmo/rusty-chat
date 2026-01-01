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
pub struct AddAdminPayload {
    pub user_id: i64,
}

#[derive(Debug, Serialize)]
pub struct Response {
    response_message: String,
    error: Option<String>,
}

pub async fn add_room_admin(
    State(state): State<AppState>,
    Path(room_id): Path<i64>,
    Json(payload): Json<AddAdminPayload>,
) -> impl IntoResponse {
    // Check if user is already a member?
    let user_exists = sqlx::query(
        r#"
        SELECT 1
        FROM users
        WHERE id = $1
        "#
    )
    .bind(payload.user_id)
    .fetch_one(&state.db)
    .await;

    if user_exists.is_err() {
        return (
            StatusCode::NOT_FOUND,
            Json(Response {
                response_message: format!("User with id: '{}' not found od does not exist", payload.user_id),
                error: Some("Member does not exist or room not found".into()),
            }),
        );
    }

    let room_exists = sqlx::query(
        r#"
        SELECT 1
        FROM rooms
        WHERE id = $1
        "#
    )
    .bind(room_id)
    .fetch_one(&state.db)
    .await;

    if room_exists.is_err() {
        return (
            StatusCode::NOT_FOUND,
            Json(Response {
                response_message: format!("Room with id: '{}' not found or does not exist", room_id),
                error: Some("Room not found".into()),
            }),
        );
    }

    let result = sqlx::query(
        r#"
        UPDATE room_members 
        SET role = 'admin' 
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
                        response_message: "Room admin added successfully".into(),
                        error: None,
                    }),
                )
            }
        },
        Err(e) => {
            error!("ADD ROOM ADMIN REQUEST FAILED");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Response {
                    response_message: "Failed to add room admin".into(),
                    error: Some(format!("Database error: {}", e)),
                }),
            )
        }
    }
}
