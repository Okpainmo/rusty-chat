use axum::{
    Json,
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserProfile {
    id: i64,
    full_name: String,
    email: String,
    profile_image_url: Option<String>,
    access_token: String,
    refresh_token: String,
    #[serde(skip_serializing)]
    password: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    response_message: String,
    response: Option<UserProfile>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UsersResponse {
    response_message: String,
    response: Option<Vec<UserProfile>>,
    error: Option<String>,
}

pub async fn get_all_users(Extension(db_pool): Extension<PgPool>) -> impl IntoResponse {
    let users_result = sqlx::query_as::<_, UserProfile>(
        "SELECT id, full_name, email, profile_image_url, password, access_token, refresh_token  FROM users"
    )
    .fetch_all(&db_pool)
    .await;

    match users_result {
        Ok(users) => (
            StatusCode::OK,
            Json(UsersResponse {
                response_message: "Users fetched successfully".to_string(),
                response: Some(users),
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(UsersResponse {
                response_message: "Failed to fetch users".to_string(),
                response: None,
                error: Some(format!("Database error: {}", e)),
            }),
        ),
    }
}
