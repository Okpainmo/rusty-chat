use axum::routing::patch;
use axum::{Router, middleware, routing::get};
use tower_cookies::CookieManagerLayer;
use crate::AppState;
use crate::domains::user::controllers::get_all_users::get_all_users;
use crate::domains::user::controllers::get_user::get_user;
use crate::domains::user::controllers::update_user::update_user;
use crate::middlewares::auth_access_middleware::access_middleware;
use crate::middlewares::auth_sessions_middleware::sessions_middleware;

pub fn user_routes(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/user/get-user/{user_id}", get(get_user))
        .route("/user/update-user/{user_id}", patch(update_user))
        .route("/user/get-all-users", get(get_all_users))
        .route_layer(middleware::from_fn_with_state(state.clone(), access_middleware))
        .route_layer(middleware::from_fn_with_state(state.clone(), sessions_middleware))
        .layer(CookieManagerLayer::new())
}
