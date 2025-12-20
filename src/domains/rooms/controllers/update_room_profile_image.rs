use crate::AppState;
use crate::middlewares::auth_sessions_middleware::SessionsMiddlewareOutput;
use crate::utils::file_upload_handler::upload_file;
use axum::extract::State;
use axum::extract::multipart::{Field, MultipartError};
use axum::{
    Json,
    extract::Multipart,
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::error;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Room {
    pub id: i64,
    pub room_name: Option<String>,
    pub is_group: bool,
    pub created_by: Option<i64>,
    pub bookmarked_by: Vec<i64>,
    pub archived_by: Vec<i64>,
    pub room_profile_image: Option<String>,
    pub co_member: Option<i64>, // for private rooms only
    pub co_members: Option<Vec<i64>>, // for private rooms only
}

#[derive(Debug, sqlx::FromRow)]
struct RoomLookup {
    id: i64,
    created_by: Option<i64>,
    is_group: bool,
}

#[derive(Debug, sqlx::FromRow)]
struct RoomMemberLookup {
    id: i64,
    room_id: i64,
    user_id: i64,
    role: String,
}

#[derive(Debug, Serialize)]
pub struct UpdateResponse {
    response_message: String,
    response: Option<Room>,
    error: Option<String>,
}

pub async fn update_room_profile_image(
    State(state): State<AppState>,
    Extension(session): Extension<SessionsMiddlewareOutput>,
    Path(room_id): Path<i64>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    // Verify room exists and get room details
    let room_result = sqlx::query_as::<_, RoomLookup>(
        "SELECT id, created_by, is_group FROM rooms WHERE id = $1"
    )
        .bind(room_id)
        .fetch_optional(&state.db)
        .await;

    let room = match room_result {
        Ok(Some(room)) => room,
        Ok(None) => {
            error!("ROOM PROFILE IMAGE UPDATE FAILED: ROOM NOT FOUND!");

            return (
                StatusCode::NOT_FOUND,
                Json(UpdateResponse {
                    response_message: "Room not found".to_string(),
                    response: None,
                    error: Some("Room profile image update failed".to_string()),
                }),
            );
        }
        Err(e) => {
            error!("ROOM PROFILE IMAGE UPDATE FAILED!");

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UpdateResponse {
                    response_message: "Room profile image update failed".to_string(),
                    response: None,
                    error: Some(format!("Server error: {}", e)),
                }),
            );
        }
    };

    // Check if user is a member of the room
    let member_result = sqlx::query_as::<_, RoomMemberLookup>(
        "SELECT id, room_id, user_id, role FROM room_members WHERE room_id = $1 AND user_id = $2"
    )
        .bind(room_id)
        .bind(session.user.id)
        .fetch_optional(&state.db)
        .await;

    let member = match member_result {
        Ok(Some(member)) => member,
        Ok(None) => {
            error!("UNAUTHORIZED ROOM PROFILE IMAGE UPDATE ATTEMPT!");

            return (
                StatusCode::UNAUTHORIZED,
                Json(UpdateResponse {
                    response_message: "You're not a member of this room".into(),
                    response: None,
                    error: Some("Unauthorized room update attempt".into()),
                }),
            );
        }
        Err(e) => {
            error!("ROOM PROFILE IMAGE UPDATE FAILED!");

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UpdateResponse {
                    response_message: "Room profile image update failed".to_string(),
                    response: None,
                    error: Some(format!("Server error: {}", e)),
                }),
            );
        }
    };

    // Only admin or creator can update room profile image
    if member.role != "admin" && Some(session.user.id) != room.created_by && !session.user.is_admin {
        error!("UNAUTHORIZED ROOM PROFILE IMAGE UPDATE ATTEMPT!");

        return (
            StatusCode::UNAUTHORIZED,
            Json(UpdateResponse {
                response_message: "Only room admin or creator can update room profile image".into(),
                response: None,
                error: Some("Unauthorized room update attempt".into()),
            }),
        );
    }

    let file = match multipart.next_field().await {
        Ok(Some(file)) => file,
        Ok(None) => {
            error!("FILE UPLOAD FAILED!");

            return (
                StatusCode::BAD_REQUEST,
                Json(UpdateResponse {
                    response_message: "No file provided".into(),
                    response: None,
                    error: Some("File upload failed".into()),
                }),
            );
        }
        Err(e) => {
            error!("FAILED TO EXTRACT FILE FOR UPLOAD: {}", e);

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UpdateResponse {
                    response_message: "File upload failed".into(),
                    response: None,
                    error: Some(e.to_string()),
                }),
            );
        }
    };

    let file_url = match upload_file(State(&state), file, &room_id).await {
        Ok(file_url) => file_url,
        Err(e) => {
            error!("ROOM PROFILE IMAGE UPLOAD FAILED!");

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UpdateResponse {
                    response_message: "Failed to upload room profile image".into(),
                    response: None,
                    error: Some(format!("File upload error: {}", e)),
                }),
            );
        }
    };

    let res = sqlx::query_as::<_, Room>(
        r#"
            UPDATE rooms
            SET
                room_profile_image = $1,
                updated_at = NOW()
            WHERE id = $2
            RETURNING id, room_name, is_group, created_by, bookmarked_by, archived_by, room_profile_image, co_member, co_members
            "#,
    )
        .bind(file_url)
        .bind(room_id)
        .fetch_one(&state.db)
        .await;

    match res {
        Ok(updated_room) => (
            StatusCode::OK,
            Json(UpdateResponse {
                response_message: "Room profile image updated successfully".into(),
                response: Some(updated_room),
                error: None,
            }),
        ),
        Err(e) => {
            error!("FAILED TO UPDATE ROOM PROFILE IMAGE!");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UpdateResponse {
                    response_message: "Failed to update room profile image".into(),
                    response: None,
                    error: Some(format!("Server error: {}", e)),
                }),
            )
        }
    }
}