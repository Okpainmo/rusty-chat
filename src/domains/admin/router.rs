use crate::AppState;
use crate::domains::admin::controllers::activate_user::activate_user;
use crate::domains::admin::controllers::add_admin::add_admin;
use crate::domains::admin::controllers::deactivate_user::deactivate_user;
use crate::domains::admin::controllers::remove_admin::remove_admin;
use crate::middlewares::admin_routes_protector::admin_routes_protector;
use crate::middlewares::auth_access_middleware::access_middleware;
use crate::middlewares::auth_sessions_middleware::sessions_middleware;
use axum::routing::patch;
use axum::{Router, middleware, routing::get};
use tower_cookies::CookieManagerLayer;

pub fn admin_routes(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/add-admin/{user_id}", patch(add_admin))
        .route("/remove-admin/{user_id}", patch(remove_admin))
        .route("/activate-user/{user_id}", patch(activate_user))
        .route("/deactivate-user/{user_id}", patch(deactivate_user))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            access_middleware,
        ))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            sessions_middleware,
        ))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            admin_routes_protector,
        ))
        .layer(CookieManagerLayer::new())
}
