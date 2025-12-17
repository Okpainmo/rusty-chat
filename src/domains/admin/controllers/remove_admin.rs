use crate::utils::generate_tokens::{User, generate_tokens};
use axum::{Json, extract::Query, http::StatusCode, response::IntoResponse};
use axum::extract::{Path, State};
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
    last_seen: Option<String>
}

pub async fn remove_admin(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    cookies: Cookies,
) -> impl IntoResponse {

    let result = sqlx::query_as::<_, UserProfile>(
        r#"
        UPDATE users
        SET
            is_admin = false,
            updated_at = NOW()
        WHERE id = $1
        RETURNING
            id,
            full_name,
            email,
            profile_image,
            status,
            last_seen,
            is_admin,
            is_active
        "#,
    )
        .bind(user_id)
        .fetch_one(&state.db)
        .await;

    match result {
        Ok(user) => {
            (
                StatusCode::OK,
                Json(LogoutResponse {
                    response_message: "Admin access revoked successfully".to_string(),
                    error: None,
                    response: Some(user),
                }),
            )
        }
        Err(e) => {
            error!("FAILED TO REVOKE ADMIN ACCESS!");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LogoutResponse {
                    response_message: "Failed to revoke admin access".to_string(),
                    error: Some(e.to_string()),
                    response: None,
                }),
            )
        }
    }
}