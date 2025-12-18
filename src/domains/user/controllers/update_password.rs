use crate::domains::auth::controllers::login_user::LoginResponse;
use crate::domains::auth::controllers::register_user::RegisterResponse;
use crate::middlewares::auth_access_middleware::ErrorResponse;
use crate::middlewares::auth_access_middleware::SessionInfo;
use crate::middlewares::auth_sessions_middleware::SessionsMiddlewareOutput;
use crate::utils::cookie_deploy_handler::deploy_auth_cookie;
use crate::utils::generate_tokens::{User, generate_tokens};
use crate::utils::hashing_handler::hashing_handler;

use crate::AppState;
use crate::utils::verification_handler::verification_handler;
use axum::extract::State;
use axum::{
    Json,
    extract::Multipart,
    extract::{Extension, Path, Request},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tower_cookies::Cookies;
use tracing::error;

#[derive(Debug, Deserialize)]
pub struct UpdateUserPayload {
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
}

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
}

#[derive(Debug, sqlx::FromRow)]
struct UserLookup {
    id: i64,
    email: String,
}

#[derive(Debug, Serialize)]
pub struct UpdateResponse {
    response_message: String,
    response: Option<UserProfile>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePasswordPayload {
    pub old_password: String,
    pub new_password: String,
}

pub async fn update_password(
    State(state): State<AppState>,
    Extension(session): Extension<SessionsMiddlewareOutput>,
    Path(user_id): Path<i64>,
    Json(payload): Json<UpdatePasswordPayload>,
) -> impl IntoResponse {
    let user = match sqlx::query_as::<_, UserProfile>(
        r#"
        SELECT id, full_name, email, profile_image, password,
               access_token, refresh_token, status, last_seen,
               is_active, is_admin
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(UpdateResponse {
                    response_message: "User not found".into(),
                    response: None,
                    error: Some("No user with this id".into()),
                }),
            );
        }
        Err(e) => {
            error!("FAILED TO FETCH USER FOR PASSWORD UPDATE!");

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UpdateResponse {
                    response_message: "Failed to update password".into(),
                    response: None,
                    error: Some(format!("Database error: {}", e)),
                }),
            );
        }
    };

    if session.user.email != user.email && !session.user.is_admin {
        error!("UNAUTHORIZED PASSWORD UPDATE ATTEMPT!");

        return (
            StatusCode::UNAUTHORIZED,
            Json(UpdateResponse {
                response_message: "You're not permitted to perform this action".into(),
                response: None,
                error: Some("Unauthorized password update attempt".into()),
            }),
        );
    }

    let password_matches = match verification_handler(&payload.old_password, &user.password).await {
        Ok(valid) => valid,
        Err(e) => {
            error!("PASSWORD VERIFICATION ERROR!");

            return (
                StatusCode::BAD_REQUEST,
                Json(UpdateResponse {
                    response_message: "Password verification failed".into(),
                    response: None,
                    error: Some(format!("Password verification error: {}", e)),
                }),
            );
        }
    };

    if !password_matches {
        return (
            StatusCode::UNAUTHORIZED,
            Json(UpdateResponse {
                response_message: "Invalid old password".into(),
                response: None,
                error: Some("Old password does not match".into()),
            }),
        );
    }

    let hashed_password = match hashing_handler(&payload.new_password).await {
        Ok(hash) => hash,
        Err(e) => {
            error!("PASSWORD HASHING ERROR!");

            return (
                StatusCode::BAD_REQUEST,
                Json(UpdateResponse {
                    response_message: "Failed to hash password".into(),
                    response: None,
                    error: Some(format!("Password hashing error: {}", e)),
                }),
            );
        }
    };

    let updated_user = match sqlx::query_as::<_, UserProfile>(
        r#"
        UPDATE users
        SET password = $1, updated_at = NOW()
        WHERE id = $2
        RETURNING id, full_name, email, profile_image, password,
                  access_token, refresh_token, status, last_seen,
                  is_active, is_admin
        "#,
    )
    .bind(hashed_password)
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    {
        Ok(user) => user,
        Err(e) => {
            error!("FAILED TO UPDATE PASSWORD!");

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UpdateResponse {
                    response_message: "Failed to update password".into(),
                    response: None,
                    error: Some(format!("Database error: {}", e)),
                }),
            );
        }
    };

    (
        StatusCode::OK,
        Json(UpdateResponse {
            response_message: "Password updated successfully".into(),
            response: Some(updated_user),
            error: None,
        }),
    )
}
