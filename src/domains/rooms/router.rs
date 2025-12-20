use crate::AppState;
use crate::domains::rooms::controllers::create_private_chat_room::create_room;
use crate::domains::rooms::controllers::create_group_chat_room::create_group;
use crate::domains::rooms::controllers::update_room_profile_image::update_room_profile_image;

use crate::middlewares::auth_access_middleware::access_middleware;
use crate::middlewares::auth_sessions_middleware::sessions_middleware;
use axum::routing::{patch, post};
use axum::{Router, middleware, routing::get};
use tower_cookies::CookieManagerLayer;

pub fn rooms_routes(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/create-private-chat-room", post(create_room))
        .route("/create-group-chat-room", post(create_group))
        .route("/update-room-profile-image/{room_id}", patch(update_room_profile_image))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            access_middleware,
        ))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            sessions_middleware,
        ))
        .layer(CookieManagerLayer::new())
}
