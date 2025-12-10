use crate::middlewares::auth_access_middleware::ErrorResponse;
use crate::middlewares::auth_access_middleware::SessionInfo;
use axum::extract::Request;
use axum::{
    Json,
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
// use crate::middlewares::auth_sessions_middleware::SessionUser;

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

pub async fn get_user(
    // Extension(session): Extension<SessionUser>,
    Extension(db_pool): Extension<PgPool>,
    Path(user_id): Path<i64>,
    // req: Request,
) -> impl IntoResponse {
    // let access_middleware_output = req
    //     .extensions()
    //     .get::<SessionInfo>()
    //     // .cloned()
    //     .ok_or_else(|| {
    //         (
    //             StatusCode::NOT_FOUND,
    //             Json(ErrorResponse {
    //                 error: "Not Found".to_string(),
    //                 response_message: "_ User not received from sessions middleware".to_string(),
    //             }),
    //         )
    //     }).unwrap()
    //     .clone();
    //
    // println!("Data received via the sessions and then the access middlewares: {:?}", access_middleware_output);

    let user_result = sqlx::query_as::<_, UserProfile>(
        "SELECT id, full_name, email, profile_image_url, password, access_token, refresh_token FROM users WHERE id = $1"
    )
        .bind(user_id)
        .fetch_optional(&db_pool)
        .await;

    match user_result {
        Ok(Some(user)) => (
            StatusCode::OK,
            Json(UserResponse {
                response_message: "User fetched successfully".to_string(),
                response: Some(user),
                error: None,
            }),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(UserResponse {
                response_message: "User not found".to_string(),
                response: None,
                error: Some(format!("No user with id {}", user_id)),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(UserResponse {
                response_message: "Failed to fetch user".to_string(),
                response: None,
                error: Some(format!("Database error: {}", e)),
            }),
        ),
    }
}
