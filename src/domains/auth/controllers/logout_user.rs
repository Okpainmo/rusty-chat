use crate::utils::generate_tokens::{User, generate_tokens};
use axum::{Json, extract::Query, http::StatusCode, response::IntoResponse};
use axum::extract::State;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tower_cookies::{Cookie, Cookies};
use tracing::{error, info};
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    response_message: String,
    response: Option<UserProfile>,
    error: Option<String>,
}

#[derive(Deserialize)]
pub struct SearchParams {
    user_email: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserProfile {
    id: i64,
    full_name: String,
    email: String,
    profile_image: Option<String>,
    is_admin: bool,
    is_active: bool,
    status: String,
    last_seen: Option<String>,
    is_logged_out: bool,
}

pub async fn logout_user(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
    cookies: Cookies,
) -> impl IntoResponse {
    // info!("Logout request for user: {}", params.user_email);

    // Remove auth cookie
    let mut cookie = Cookie::new("rusty_chat_auth_cookie", "");
    cookie.set_path("/");
    cookie.set_max_age(tower_cookies::cookie::time::Duration::ZERO);
    cookies.remove(cookie);

    // Clear tokens in database - IMPORTANT: Add RETURNING clause
    let user = sqlx::query_as::<_, UserProfile>(
        r#"
                UPDATE users
                SET
                    access_token = $1,
                    refresh_token = $2,
                    is_logged_out = $3,
                    updated_at = NOW()
                WHERE email = $4
                RETURNING
                id,
                full_name,
                is_logged_out,
                email,
                profile_image,
                status,
                last_seen,
                is_admin,
                is_active,
                is_logged_out
            "#,
    )
        .bind("") // profile_image_url
        .bind("") // profile_image_url
        .bind(true)
        .bind(&params.user_email)
        .fetch_one(&state.db)
        .await;

    match user {
        Ok(user) => {
            (
                StatusCode::OK,
                Json(LogoutResponse {
                    response_message: "Logout successful".to_string(),
                    error: None,
                    response: Some(user),
                }),
            )
        }
        Err(e) => {
            error!("USER LOGOUT WAS UNSUCCESSFUL!");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LogoutResponse {
                    response_message: "Logout failed".to_string(),
                    error: Some(e.to_string()),
                    response: None,
                }),
            )
        }
    }
}