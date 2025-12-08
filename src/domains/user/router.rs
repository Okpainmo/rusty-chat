use axum::{Router, routing::get, Extension};
use crate::domains::user::controllers::get_user::get_user;
use crate::domains::user::controllers::get_all_users::get_all_users;
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};

pub fn user_routes() -> Router {
    Router::new()
        .route("/user/get-user/{user_id}", get(get_user))
        .route("/user/get-all-users", get(get_all_users))
        .layer(CookieManagerLayer::new())
}
