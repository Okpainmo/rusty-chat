use axum::{Json, extract::Extension, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use sqlx::PgPool;
use tower_cookies::{Cookie, Cookies};

#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    response_message: String,
    error: Option<String>,
}

pub async fn logout_user(
    cookies: Cookies,
    Extension(db_pool): Extension<PgPool>,
) -> impl IntoResponse {
    // Create a new cookie with the same name and expire it
    let mut cookie = Cookie::new("rusty_chat_auth_cookie", "");
    cookie.set_path("/");
    cookie.set_max_age(time::Duration::ZERO);

    cookies.remove(cookie);

    (
        StatusCode::OK,
        Json(LogoutResponse {
            response_message: "Logout successful".to_string(),
            error: None,
        }),
    )
}