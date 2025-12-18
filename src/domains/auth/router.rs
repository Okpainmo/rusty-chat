use super::controllers::{login_user, register_user};
use crate::AppState;
use crate::domains::auth::controllers::login_user::login_user;
use crate::domains::auth::controllers::logout_user::logout_user;
use crate::domains::auth::controllers::register_user::register_user;
use axum::{Extension, Router, routing::post};
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};

pub fn auth_routes(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register_user))
        .route("/auth/login", post(login_user))
        .route("/auth/logout", post(logout_user))
        .layer(CookieManagerLayer::new())
}
