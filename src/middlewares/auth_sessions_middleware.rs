use axum::{
    Extension, Json,
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::IntoResponse,
};
use jsonwebtoken::{DecodingKey, Validation, decode, errors::ErrorKind};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tower_cookies::Cookies;

use crate::utils::cookie_deploy_handler::deploy_auth_cookie;
use crate::utils::generate_tokens::User;

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub response_message: String,
}

#[derive(Debug, Deserialize)]
pub struct JwtClaims {
    pub id: i64,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Serialize, sqlx::FromRow, Clone)]
pub struct UserProfile {
    pub id: i64,
    pub full_name: String,
    pub email: String,
    pub refresh_token: Option<String>,
    pub profile_image_url: Option<String>,
}

#[derive(Clone)]
pub struct SessionState {
    pub jwt_secret: String,
    pub cookie_name: String,
}

#[derive(Clone, Debug)]
pub struct SessionsMiddlewareOutput {
    pub user: User,
    pub session_status: String,
}

// ============================================================================
// Sessions Middleware
// ============================================================================

pub async fn sessions_middleware(
    cookies: Cookies,
    Extension(db_pool): Extension<PgPool>,
    mut req: Request,
    next: Next,
) -> impl IntoResponse {
    // println!("hello session middleware");

    let state = Arc::new(SessionState {
        jwt_secret: std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
        cookie_name: "rusty_chat_auth_cookie".to_string(),
    });

    // ------------------------------------------------------------------------
    // Extract required headers
    // ------------------------------------------------------------------------
    let email = req
        .headers()
        .get("email")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Unauthorized".to_string(),
                    response_message: "Email header missing".to_string(),
                }),
            )
        })?;

    // println!("email: { }", email);

    let authorization = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Unauthorized".to_string(),
                    response_message: "Authorization header missing".to_string(),
                }),
            )
        })?;

    // ------------------------------------------------------------------------
    // Validate cookie presence
    // ------------------------------------------------------------------------
    if cookies.get(&state.cookie_name).is_none() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Unauthorized".to_string(),
                response_message: "Request rejected, please re-authenticate".to_string(),
            }),
        ));
    }

    // ------------------------------------------------------------------------
    // Fetch user from Postgres
    // ------------------------------------------------------------------------
    let user = match sqlx::query_as::<_, UserProfile>(
        r#"
        SELECT id, full_name, email, refresh_token, profile_image_url
        FROM users
        WHERE email = $1
        "#,
    )
    .bind(&email)
    .fetch_optional(&db_pool)
    .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Not Found".to_string(),
                    response_message: format!("User '{}' not found", email),
                }),
            ));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "DB Error".to_string(),
                    response_message: e.to_string(),
                }),
            ));
        }
    };

    // Ensure refresh/session token exists
    let refresh = match &user.refresh_token {
        Some(t) => t.clone(),
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Not Found".to_string(),
                    response_message: "Refresh token missing".to_string(),
                }),
            ));
        }
    };

    // ------------------------------------------------------------------------
    // Verify REFRESH / SESSION JWT
    // ------------------------------------------------------------------------
    let decoding_key = DecodingKey::from_secret(state.jwt_secret.as_bytes());

    match decode::<JwtClaims>(&refresh, &decoding_key, &Validation::default()) {
        Ok(token_data) => {
            if token_data.claims.email != user.email {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse {
                        error: "Unauthorized".to_string(),
                        response_message: "User credentials do not match".to_string(),
                    }),
                ));
            }

            let session_status = "USER SESSION IS ACTIVE".to_string();
            // println!("{}", session_status);

            // Store session user in request extensions
            req.extensions_mut().insert(SessionsMiddlewareOutput {
                user: User {
                    id: user.id,
                    email: user.email.clone(),
                },
                session_status,
            });
        }

        Err(err) => match err.kind() {
            ErrorKind::ExpiredSignature => {
                let session_status =
                    format!("EXPIRED SESSION: session terminated for '{}'", user.email);
                println!("{}", session_status);

                return Err((
                    StatusCode::FORBIDDEN,
                    Json(ErrorResponse {
                        error: "Forbidden".to_string(),
                        response_message: "User session expired, please re-authenticate"
                            .to_string(),
                    }),
                ));
            }
            _ => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Session Verification Failed".to_string(),
                        response_message: err.to_string(),
                    }),
                ));
            }
        },
    }
    //
    // println!("{:#?}", &req);
    // println!("{:#?}", &user);
    Ok(next.run(req).await)
}
