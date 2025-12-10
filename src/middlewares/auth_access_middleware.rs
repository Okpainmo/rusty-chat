use crate::middlewares::auth_sessions_middleware::SessionsMiddlewareOutput;
use crate::utils::cookie_deploy_handler::deploy_auth_cookie;
use crate::utils::generate_tokens::{User, generate_tokens};
use axum::{
    Extension, Json,
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sqlx;
use sqlx::PgPool;
use std::sync::Arc;
use tower_cookies::{Cookie, Cookies};
// ============================================================================
// Types/Structures
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub id: i64,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Clone)]
pub struct AppState {
    pub jwt_secret: String,
    pub cookie_name: String,
    // Add database connection pool here
    // pub db: DatabasePool,
}

#[derive(Clone, Debug)]
pub struct SessionInfo {
    pub user: User,
    pub new_access_token: String,
    pub new_refresh_token: String,
    pub session_status: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserProfile {
    // #[sqlx(rename = "id")]
    id: i64,
    full_name: String,
    email: String,
    profile_image_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub response_message: String,
}

pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub auth_cookie: String,
}

enum TokenStatus {
    Valid,
    Expired,
    Invalid(String),
}

fn verify_access_token(token: &str, secret: &str, user: &User) -> TokenStatus {
    let validation = Validation::default();
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());

    match decode::<JwtClaims>(token, &decoding_key, &validation) {
        Ok(token_data) => {
            // Verify email matches
            if token_data.claims.email != user.email {
                return TokenStatus::Invalid("User credentials do not match".to_string());
            }
            TokenStatus::Valid
        }
        Err(err) => {
            use jsonwebtoken::errors::ErrorKind;
            match err.kind() {
                ErrorKind::ExpiredSignature => TokenStatus::Expired,
                _ => TokenStatus::Invalid(format!("Token verification failed: {}", err)),
            }
        }
    }
}

// ============================================================================
// Middleware Implementation
// ============================================================================

pub async fn access_middleware(
    // State(state): State<Arc<AppState>>, // see state declaration inside of main.rs
    // axum::Extension(session_info): axum::Extension<SessionInfo>,
    cookies: Cookies,
    Extension(db_pool): Extension<PgPool>,
    mut req: Request,
    next: Next,
) -> impl IntoResponse {
    // println!("hello access middleware");
    // println!("hello access middleware: {:#?}", {session_info});

    let state = Arc::new(AppState {
        jwt_secret: std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
        cookie_name: "rusty_chat_auth_cookie".to_string(),
    });

    // Check for auth cookie - reject the request immediately if auth cookie is missing
    let auth_cookie = cookies.get(&state.cookie_name).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Unauthorized".to_string(),
                response_message: "Request rejected, please re-authenticate".to_string(),
            }),
        )
    })?;

    // Get user from request extensions (should be set by session middleware)
    let sessions_middleware_output = req
        .extensions()
        .get::<SessionsMiddlewareOutput>()
        // .cloned()
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Not Found".to_string(),
                    response_message: "_ User not received from sessions middleware".to_string(),
                }),
            )
        })?
        .clone();

    // println!("session middleware output 1 {:#?}", req);

    // Extract authorization header
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::FORBIDDEN,
                Json(ErrorResponse {
                    error: "Forbidden".to_string(),
                    response_message: "Authorization header missing".to_string(),
                }),
            )
        })?;

    // Verify Bearer token format
    if !auth_header.starts_with("Bearer ") {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Forbidden".to_string(),
                response_message:
                    "Authorization string does not match expected (Bearer Token) format".to_string(),
            }),
        ));
    }

    let access_token = auth_header.trim_start_matches("Bearer ");

    // println!("session middleware output 1b {:#?}", sessions_middleware_output.user);

    let tokens = match generate_tokens(
        "auth",
        User {
            id: sessions_middleware_output.user.id,
            email: sessions_middleware_output.user.email.clone(),
        },
    )
    .await
    {
        Ok(tokens) => tokens,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    response_message: "Failed to generate tokens".to_string(),
                    error: format!("Token generation error: {}", e.to_string()),
                }),
            ));
        }
    };

    // Verify and process access token
    match verify_access_token(
        access_token,
        &state.jwt_secret,
        &sessions_middleware_output.user,
    ) {
        TokenStatus::Valid => {
            /* Token is valid, renew tokens */

            // Update user in database
            let _ = sqlx::query_as::<_, UserProfile>(
                r#"
                    UPDATE users
                    SET
                        access_token = $1,
                        refresh_token = $2,
                        updated_at = NOW()
                    WHERE email = $3
                "#,
            )
            .bind(&tokens.access_token)
            .bind(&tokens.refresh_token)
            .bind(&sessions_middleware_output.user.email)
            .fetch_one(&db_pool)
            .await;

            // Deploy new cookie
            deploy_auth_cookie(cookies, tokens.auth_cookie.unwrap()).await;

            // Store session info in request extensions
            {
                req.extensions_mut().insert(SessionInfo {
                    user: sessions_middleware_output.user.clone(),
                    new_access_token: tokens.access_token.unwrap().to_string(),
                    new_refresh_token: tokens.refresh_token.unwrap().to_string(),
                    session_status: format!(
                        "ACTIVE ACCESS WITH ACTIVE SESSION: access and session renewed for '{}'",
                        sessions_middleware_output.user.email
                    ),
                });
            }

            // println!("Active access: tokens renewed for '{}'", sessions_middleware_output.user.email);
        }

        TokenStatus::Expired => {
            /* The fact that the request passes the session middleware that is placed before this access middleware,
            confirms that the session is still valid even though the access token is currently expired. Hence, we renew the
            tokens */

            // Update user in database
            let _ = sqlx::query_as::<_, UserProfile>(
                r#"
                    UPDATE users
                    SET
                        access_token = $1,
                        refresh_token = $2,
                        updated_at = NOW()
                    WHERE email = $3
                "#,
            )
            .bind(&tokens.access_token)
            .bind(&tokens.refresh_token)
            .bind(&sessions_middleware_output.user.email)
            .fetch_one(&db_pool)
            .await;

            // Deploy new cookie
            deploy_auth_cookie(cookies, tokens.auth_cookie.unwrap()).await;

            // Store session info in request extensions
            {
                req.extensions_mut().insert(SessionInfo {
                    user: sessions_middleware_output.user.clone(),
                    new_access_token: tokens.access_token.unwrap().to_string(),
                    new_refresh_token: tokens.refresh_token.unwrap().to_string(),
                    session_status: format!(
                        "EXPIRED ACCESS WITH ACTIVE SESSION: access and session renewed for '{}'",
                        sessions_middleware_output.user.email
                    ),
                });
            }

            // println!("Expired access: tokens renewed for '{}'", sessions_middleware_output.user.email);
        }
        TokenStatus::Invalid(msg) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Unauthorized".to_string(),
                    response_message: msg,
                }),
            ));
        }
    }

    // println!("session middleware output 2 {:#?}", req);

    Ok(next.run(req).await)
}
