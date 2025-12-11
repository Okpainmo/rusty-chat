use crate::domains::auth::controllers::login_user::{LoginResponse, ResponseCore, UserProfile};
use crate::utils::generate_tokens::{User, generate_tokens};
use axum::{Json, extract::Extension, extract::Query, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tower_cookies::{Cookie, Cookies};
use tracing::{error, info};

#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    response_message: String,
    response: Option<ResponseCore>,
    error: Option<String>,
}

#[derive(Deserialize)]
pub struct SearchParams {
    user_email: String,
}

pub async fn logout_user(
    Query(params): Query<SearchParams>,
    cookies: Cookies,
    Extension(db_pool): Extension<PgPool>,
) -> impl IntoResponse {
    // info!("Query param: {}!", params.user_email.to_string());

    // Create a new cookie with the same name and expire it
    let mut cookie = Cookie::new("rusty_chat_auth_cookie", "");
    cookie.set_path("/");
    cookie.set_max_age(time::Duration::ZERO);

    cookies.remove(cookie);

    let user = sqlx::query_as::<_, UserProfile>(
        r#"
                        UPDATE users
                        SET
                            access_token = $1,
                            refresh_token = $2,
                            updated_at = NOW()
                        WHERE email = $3
                    "#,
    )
    .bind("") // profile_image_url
    .bind("") // profile_image_url
    .bind(&params.user_email)
    .fetch_one(&db_pool)
    .await;

    match user {
        Ok(user) => {
            (
                StatusCode::OK,
                Json(LogoutResponse {
                    response_message: "Logout successful".to_string(),
                    error: None,
                    response: None,
                }),
            )
        },
        Err(e) =>  {
            error!("USER LOGOUT WAS UNSUCCESSFUL!");
            
            (
                StatusCode::OK,
                Json(LogoutResponse {
                response_message: "Logout failed!".to_string(),
                error: None,
                response: None,
                }),
            )
        }
    }

}
