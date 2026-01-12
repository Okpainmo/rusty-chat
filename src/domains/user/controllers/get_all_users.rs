use crate::AppState;
use axum::extract::State;
use axum::{Json, http::StatusCode, response::IntoResponse};
use chrono::NaiveDateTime;
use serde::Serialize;
use tracing::error;
// use crate::middlewares::auth_sessions_middleware::SessionUser;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserProfile {
    id: i64,
    full_name: String,
    email: String,
    profile_image: Option<String>,
    access_token: String,
    refresh_token: String,
    status: String,
    last_seen: Option<String>,
    is_admin: bool,
    is_active: bool,
    country: String,
    phone_number: String,
    is_logged_out: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct OutputCore {
    count: usize,
    users: Option<Vec<UserProfile>>,
}

#[derive(Debug, Serialize)]
pub struct UsersResponse {
    response_message: String,
    response: Option<OutputCore>,
    error: Option<String>,
}

pub async fn get_all_users(
    // Extension(db_pool): Extension<PgPool>,
    // req: Request
    State(state): State<AppState>,
) -> impl IntoResponse {
    let users_result = sqlx::query_as::<_, UserProfile>(
        "SELECT id, full_name, email, profile_image, access_token, refresh_token, status, last_seen, is_active, is_admin, country, phone_number, is_logged_out, created_at, updated_at FROM users"
    )
    .fetch_all(&state.db)
    .await;

    match users_result {
        Ok(users) => (
            StatusCode::OK,
            Json(UsersResponse {
                response_message: "Users fetched successfully".to_string(),
                response: Some(OutputCore {
                    count: users.len(),
                    users: Some(users),
                }),
                error: None,
            }),
        ),
        Err(e) => {
            error!("FAILED TO FETCH USERS!");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UsersResponse {
                    response_message: "Failed to fetch users".to_string(),
                    response: None,
                    error: Some(format!("Database error: {}", e)),
                }),
            )
        }
    }
}
