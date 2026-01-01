use crate::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
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
pub struct RoomsResponse {
    response_message: String,
    count: usize,
    response: Vec<Room>,
    error: Option<String>,
}

pub async fn get_all_group_rooms(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Room>("SELECT * FROM rooms WHERE is_group = true")
        .fetch_all(&state.db)
        .await;

    match result {
        Ok(rooms) => (
            StatusCode::OK,
            Json(RoomsResponse {
                response_message: "Public rooms retrieved successfully".into(),
                count: rooms.len(),
                response: rooms,
                error: None,
            }),
        ),
        Err(e) => {
            error!("FETCH PUBLIC ROOMS REQUEST FAILED");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RoomsResponse {
                    response_message: "Failed to retrieve public rooms".into(),
                    count: 0,
                    response: vec![],
                    error: Some(format!("Database error: {}", e)),
                }),
            )
        }
    }
}
