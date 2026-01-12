use crate::AppState;
use crate::middlewares::auth_sessions_middleware::SessionsMiddlewareOutput;
use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::NaiveDateTime;
use serde::Serialize;
use tracing::error;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Room {
    pub id: i64,
    pub room_name: Option<String>,
    pub is_group: bool,
    pub created_by: Option<i64>,
    pub bookmarked_by: Vec<i64>,
    pub archived_by: Vec<i64>,
    pub room_profile_image: Option<String>,
    pub co_member: Option<i64>,
    pub co_members: Option<Vec<i64>>,
    pub is_public: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct RoomResponse {
    response_message: String,
    response: Option<Room>,
    error: Option<String>,
}

pub async fn get_room(
    State(state): State<AppState>,
    Extension(_session): Extension<SessionsMiddlewareOutput>,
    Path(room_id): Path<i64>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Room>("SELECT * FROM rooms WHERE id = $1")
        .bind(room_id)
        .fetch_optional(&state.db)
        .await;

    match result {
        Ok(Some(room)) => (
            StatusCode::OK,
            Json(RoomResponse {
                response_message: "Room retrieved successfully".into(),
                response: Some(room),
                error: None,
            }),
        ),
        Ok(None) => {
            error!("FAILED TO FETCH ROOM: ROOM NOT FOUND!");
            (
                StatusCode::NOT_FOUND,
                Json(RoomResponse {
                    response_message: "Room not found or does not exist".into(),
                    response: None,
                    error: Some("Room with the provided ID does not exist".into()),
                }),
            )
        }
        Err(e) => {
            error!("FAILED TO FETCH ROOM: DATABASE ERROR!");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RoomResponse {
                    response_message: "Failed to retrieve room".into(),
                    response: None,
                    error: Some(format!("Database error: {}", e)),
                }),
            )
        }
    }
}
