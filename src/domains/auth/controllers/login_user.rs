use axum::{Json, extract::Extension, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use crate::domains::auth::controllers::register_user::RegisterResponse;
use crate::utils::generate_tokens::{generate_tokens, User};
// utils import
use crate::utils::verification_handler::verification_handler; // your existing password verification function
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
use crate::utils::cookie_deploy_handler::deploy_auth_cookie;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserProfile {
    #[sqlx(rename = "id")]
    user_id: i64,
    full_name: String,
    email: String,
    profile_image_url: Option<String>,
    #[serde(skip_serializing)]
    password: String,
}

#[derive(Debug, Serialize)]
pub struct ResponseCore {
    user_profile: UserProfile,
    access_token: Option<String>,
    refresh_token:  Option<String>
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    response_message: String,
    response: Option<ResponseCore>,
    error: Option<String>,
}

// Reuse UserProfile and ResponseCore from register controller

pub async fn login_user(
    cookies: Cookies,
    Extension(db_pool): Extension<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    // Fetch user by email
    let user_result = sqlx::query_as::<_, UserProfile>(
        "SELECT id, full_name, email, profile_image_url, password FROM users WHERE email = $1",
    )
    .bind(&payload.email)
    .fetch_optional(&db_pool)
    .await;

    let user = match user_result {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(LoginResponse {
                    response_message: "Login failed".to_string(),
                    response: None,
                    error: Some("Invalid email or password".to_string()),
                }),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LoginResponse {
                    response_message: "Login failed".to_string(),
                    response: None,
                    error: Some(format!("Database error: {}", e)),
                }),
            );
        }
    };

    // Verify password using your custom handler
    match verification_handler(&payload.password, &user.password).await {
        Ok(true) => {
            let tokens = match generate_tokens("auth", User { id: 3, email: payload.email.clone() }).await {
                Ok(tokens) => tokens,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(LoginResponse {
                            response_message: "Failed to generate tokens".to_string(),
                            response: None,
                            error: Some(format!("Token generation error: {}", e)),
                        }),
                    );
                }
            };

            deploy_auth_cookie(cookies, tokens.auth_cookie.unwrap()).await;

            (
                StatusCode::OK,
                Json(LoginResponse {
                    response_message: "Login successful".to_string(),
                    response: Some(ResponseCore
                    {
                        user_profile: user,
                        access_token: tokens.access_token,
                        refresh_token: tokens.refresh_token
                        }),
                    error: None,
                }),
            )
        } ,
        Ok(false) => (
            StatusCode::UNAUTHORIZED,
            Json(LoginResponse {
                response_message: "Login failed".to_string(),
                response: None,
                error: Some("Invalid email or password".to_string()),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(LoginResponse {
                response_message: "Login failed".to_string(),
                response: None,
                error: Some(format!("Password verification error: {}", e)),
            }),
        ),
    }
}
