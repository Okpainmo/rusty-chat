use axum::{Router, middleware, routing::get};
use tower_cookies::CookieManagerLayer;

use crate::domains::user::controllers::get_all_users::get_all_users;
use crate::domains::user::controllers::get_user::get_user;
use crate::middlewares::auth_access_middleware::access_middleware;
use crate::middlewares::auth_sessions_middleware::sessions_middleware;

pub fn user_routes() -> Router {
    Router::new()
        .route("/user/get-user/{user_id}", get(get_user))
        .route("/user/get-all-users", get(get_all_users))
        .route_layer(middleware::from_fn(access_middleware))
        .route_layer(middleware::from_fn(sessions_middleware))
        .layer(CookieManagerLayer::new())
}
