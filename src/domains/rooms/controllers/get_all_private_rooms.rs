use crate::AppState;
// use crate::middlewares::auth_sessions_middleware::SessionsMiddlewareOutput;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
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
    pub pinned_by: Vec<i64>,
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

pub async fn get_all_private_rooms(
    State(state): State<AppState>,
    // Extension(session): Extension<SessionsMiddlewareOutput>,
) -> impl IntoResponse {
    // let user_id = session.user.id;
    // let user_exists = sqlx::query(
    //     r#"
    //     SELECT 1
    //     FROM users
    //     WHERE id = $1
    //     "#
    // )
    // .bind(user_id)
    // .fetch_one(&state.db)
    // .await;

    // if user_exists.is_err() {
    //     return (
    //         StatusCode::NOT_FOUND,
    //         Json(Response {
    //             response_message: format!("User with id: '{}' not found od does not exist", user_id),
    //             error: Some("Member does not exist or room not found".into()),
    //         }),
    //     );
    // }

    // Fetch rooms that are NOT public AND the user is a member of
    let result = sqlx::query_as::<_, Room>(
        r#"
            SELECT * FROM rooms WHERE is_group = false
       "#,
    )
    .fetch_all(&state.db)
    .await;

    match result {
        Ok(rooms) => (
            StatusCode::OK,
            Json(RoomsResponse {
                response_message: "Private rooms retrieved successfully".into(),
                count: rooms.len(),
                response: rooms,
                error: None,
            }),
        ),
        Err(e) => {
            error!("FETCH PRIVATE ROOMS REQUEST FAILED");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RoomsResponse {
                    response_message: "Failed to retrieve private rooms".into(),
                    count: 0,
                    response: vec![],
                    error: Some(format!("Database error: {}", e)),
                }),
            )
        }
    }
}
