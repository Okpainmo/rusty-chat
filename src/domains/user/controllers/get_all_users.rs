use crate::AppState;
use axum::extract::State;
use axum::{
    Json,
    http::StatusCode,
    response::IntoResponse,
};
use serde::Serialize;
use tracing::error;
use chrono::NaiveDateTime;
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
    #[serde(skip_serializing)]
    password: String,
    is_admin: bool,
    is_active: bool,
    country: String,
    phone_number: String,
    is_logged_out: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
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

pub async fn get_all_users(
    // Extension(db_pool): Extension<PgPool>,
    // req: Request
    State(state): State<AppState>,
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

    let users_result = sqlx::query_as::<_, UserProfile>(
        "SELECT id, full_name, email, profile_image, password, access_token, refresh_token, status, last_seen, is_active, is_admin, country, phone_number, is_logged_out, created_at, updated_at FROM users"
    )
    .fetch_all(&state.db)
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
