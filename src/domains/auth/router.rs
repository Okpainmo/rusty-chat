use axum::{Router, routing::post, Extension};
use crate::domains::auth::controllers::login_user::login_user;
use crate::domains::auth::controllers::register_user::register_user;
use super::controllers::{login_user, register_user};
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
use crate::domains::auth::controllers::logout_user::logout_user;

pub fn auth_routes() -> Router {
    Router::new()
        .route("/auth/register", post(register_user))
        .route("/auth/login", post(login_user))
        .route("/auth/logout", post(logout_user))
        .layer(CookieManagerLayer::new())
}
