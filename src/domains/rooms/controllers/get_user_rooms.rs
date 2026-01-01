use crate::AppState;
// use crate::middlewares::auth_sessions_middleware::SessionsMiddlewareOutput;
use axum::{
    extract::{Path, State},
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

pub async fn get_user_rooms(
    State(state): State<AppState>,
    // Extension(session): Extension<SessionsMiddlewareOutput>,
    Path(user_id): Path<i64>,
) -> impl IntoResponse {
    // let user_id = session.user.id;

    let result = sqlx::query_as::<_, Room>(
        r#"
        SELECT r.* 
        FROM rooms r
        INNER JOIN room_members rm ON r.id = rm.room_id
        WHERE rm.user_id = $1
        "#
    )
        .bind(user_id)
        .fetch_all(&state.db)
        .await;

    match result {
        Ok(rooms) => (
            StatusCode::OK,
            Json(RoomsResponse {
                response_message: "User rooms retrieved successfully".into(),
                count: rooms.len(),
                response: rooms,
                error: None,
            }),
        ),
        Err(e) => {
            error!("FETCH USER ROOMS REQUEST FAILED");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RoomsResponse {
                    response_message: "Failed to retrieve user rooms".into(),
                    count: 0,
                    response: vec![],
                    error: Some(format!("Database error: {}", e)),
                }),
            )
        }
    }
}
