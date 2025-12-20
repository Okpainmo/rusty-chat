use crate::utils::generate_tokens::User;
use crate::utils::generate_tokens::generate_tokens;
use axum::extract::State;
use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use tracing::error;
use crate::AppState;
use crate::utils::cookie_deploy_handler::deploy_auth_cookie;
use crate::utils::hashing_handler::hashing_handler;
use tower_cookies::Cookies;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    first_name: String,
    last_name: String,
    email: String,
    password: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserProfile {
    id: i64,
    full_name: String,
    email: String,
    profile_image: String,
}

#[derive(Debug, Serialize)]
pub struct ResponseCore {
    user_profile: UserProfile,
    access_token: Option<String>,
    refresh_token: Option<String>,
}

// ====== Response Data ======
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    response_message: String,
    response: Option<ResponseCore>,
    error: Option<String>,
}

pub async fn register_user(
    cookies: Cookies,
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    // Hash the password
    let hashed_password = match hashing_handler(payload.password.as_str()).await {
        Ok(hash) => hash,
        Err(e) => {
            error!("PASSWORD HASHING ERROR!");

            return (
                StatusCode::BAD_REQUEST,
                Json(RegisterResponse {
                    response_message: "Failed to hash password".to_string(),
                    response: None,
                    error: Some(format!("Password hashing error: {}", e)),
                }),
            );
        }
    };

    // Check if email already exists
    let email_exists: Option<i64> =
        sqlx::query_scalar("SELECT 1 FROM users WHERE email = $1 LIMIT 1;")
            .bind(&payload.email)
            .fetch_optional(&state.db)
            .await
            .unwrap_or(None)
            .flatten();

    if email_exists.unwrap_or(0) > 0 {
        error!("REGISTRATION FAILED: USER WITH EMAIL ALREADY EXIST!");

        return (
            StatusCode::FORBIDDEN,
            Json(RegisterResponse {
                response_message: "Registration failed".to_string(),
                response: None,
                error: Some("Email already exists".to_string()),
            }),
        );
    }

    let full_name = format!("{} {}", payload.first_name, payload.last_name);

    // Create user
    let result = sqlx::query_as::<_, UserProfile>(
        r#"
        INSERT INTO users (
            email,
            password,
            full_name,
            profile_image
        )
        VALUES ($1, $2, $3, $4)
        RETURNING
            id,
            full_name,
            email,
            profile_image
        "#,
    )
        .bind(&payload.email)
        .bind(&hashed_password)
        .bind(&full_name)
        .bind("")
        .fetch_one(&state.db)
        .await;

    match result {
        Ok(new_user) => {
            let tokens = match generate_tokens(
                "auth",
                User {
                    id: new_user.id,
                    email: payload.email.clone(),
                },
            )
                .await
            {
                Ok(tokens) => tokens,
                Err(e) => {
                    error!("TOKEN GENERATION ERROR!");

                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(RegisterResponse {
                            response_message: "Failed to generate tokens".to_string(),
                            response: None,
                            error: Some(format!("Token generation error: {}", e)),
                        }),
                    );
                }
            };

            // Update tokens for the created user
            let update_result = sqlx::query(
                r#"
                UPDATE users
                SET
                    access_token = $1,
                    refresh_token = $2
                WHERE id = $3
                "#,
            )
                .bind(&tokens.access_token)
                .bind(&tokens.refresh_token)
                .bind(new_user.id)
                .execute(&state.db)
                .await;

            if let Err(e) = update_result {
                error!("FAILED TO UPDATE TOKENS: {}", e);
            }

            deploy_auth_cookie(cookies, tokens.auth_cookie.unwrap()).await;

            (
                StatusCode::CREATED,
                Json(RegisterResponse {
                    response_message: format!(
                        "User with email '{}' registered successfully!",
                        &payload.email
                    ),
                    response: Some(ResponseCore {
                        user_profile: new_user,
                        access_token: tokens.access_token,
                        refresh_token: tokens.refresh_token,
                    }),
                    error: None,
                }),
            )
        }
        Err(e) => {
            let error_msg =
                if e.to_string().contains("unique") || e.to_string().contains("duplicate") {
                    error!("REGISTRATION FAILED: USER WITH EMAIL ALREADY EXIST!");
                    "Email already exists".to_string()
                } else {
                    error!("REGISTRATION FAILED: AN ERROR OCCURRED WHILE REGISTERING NEW USER!");
                    format!("Database error: {}", e)
                };

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegisterResponse {
                    response_message: "Failed to register user".to_string(),
                    response: None,
                    error: Some(error_msg),
                }),
            )
        }
    }
}
