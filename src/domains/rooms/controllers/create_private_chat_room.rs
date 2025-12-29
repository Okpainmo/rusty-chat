use crate::AppState;
use crate::middlewares::auth_sessions_middleware::SessionsMiddlewareOutput;
use axum::extract::Query;
use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::error;
use chrono::NaiveDateTime;

#[derive(Debug, Deserialize)]
pub struct CreateRoomPayload {
    // pub room_name: String,
    pub co_member: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserLookUp {
    id: i64,
    full_name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Room {
    pub id: i64,
    pub room_name: Option<String>,
    pub is_group: bool,
    pub created_by: Option<i64>,
    pub bookmarked_by: Vec<i64>,
    pub archived_by: Vec<i64>,
    pub co_member: i64, // for private rooms only
    pub is_public: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RoomMember {
    pub id: i64,
    pub room_id: i64,
    pub user_id: i64,
    pub role: String,
    pub joined_at: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct Response {
    pub response_message: String,
    pub response: Option<Room>,
    pub error: Option<String>,
}

// #[derive(Deserialize)]
// pub struct SearchParams {
//     chat_type: String,
// }

fn current_time_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Failed to evaluate time in milliseconds!")
        .as_millis()
}

pub async fn create_room(
    State(state): State<AppState>,
    // Query(params): Query<SearchParams>,
    Extension(session): Extension<SessionsMiddlewareOutput>,
    Json(payload): Json<CreateRoomPayload>,
) -> impl IntoResponse {
    println!("Time in milliseconds {:?}", current_time_millis());

    let created_by = session.user.id;

    if payload.co_member == created_by {
        error!("ROOM CREATION ERROR: SELF ROOM CREATION ATTEMPT!");

        return (
            StatusCode::NOT_FOUND,
            Json(Response {
                response_message: "Self room creation not permitted!".to_string(),
                error: Some("Room creation error".to_string()),
                response: None,
            }),
        )
    }

    let co_member =  match sqlx::query_as::<_, UserLookUp>(
        "SELECT id, full_name, created_at, updated_at FROM users WHERE id = $1",
    )
        .bind(&payload.co_member)
        .fetch_optional(&state.db)
        .await
        {
            Ok(Some(member)) => member,
            Ok(None) => {
                error!("ROOM CREATION ERROR: CO-MEMBER DATA NOT PROVIDED!");

                return (
                    StatusCode::NOT_FOUND,
                    Json(Response {
                        response_message: format!("Co-member with id: '{}' not found", payload.co_member),
                        error: Some("Room creation error".to_string()),
                        response: None,
                    }),
                )
            },
            Err(e) => {
                error!("ROOM CREATION ERROR!");

                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(Response {
                        response_message: format!("Co-member with id: '{}' not found", payload.co_member),
                        error: Some(format!("Room creation error: {:?}", e)),
                        response: None,
                    })
                )
            }
        };

    // check to prevent creating a duplicate room for the same private chat
    match sqlx::query_as::<_, Room>(
        "SELECT id, room_name, is_group, created_by, bookmarked_by, archived_by, co_member, is_public, created_at, updated_at
                FROM rooms
                WHERE created_by = $1
                    AND room_name = $2
                    AND is_group =  $3
            ",
    )
        .bind(&created_by)
        .bind(&co_member.full_name)
        .bind(false)
        .fetch_all(&state.db)
        .await
         {
             Ok(rooms) => {
                 /* a room can have the same room name since multiple users can have the same names
                  but their ids is certainly what must differ */
                 for room in rooms {
                     if(room.co_member == co_member.id) {
                         error!("DUPLICATE PRIVATE CHAT ROOM CREATION ATTEMPT!");

                         return (
                             StatusCode::BAD_REQUEST,
                             Json(Response {
                                 response_message: "Room already exists".into(),
                                 response: None,
                                 error: Some("Cannot create duplicate 1-on-1 room".into()),
                             }),
                         );
                     }
                 }
             },
             Err(e) => {
                 error!("ROOM CREATION ERROR!");

                 return (
                     StatusCode::BAD_REQUEST,
                     Json(Response {
                         response_message: "Room creation error".into(),
                         response: None,
                         error: Some(format!("Room creation error: {}", e)),
                     }),
                 );
             }
        };

    let room = match sqlx::query_as::<_, Room>(
        r#"
        INSERT INTO rooms (room_name, is_group, created_by, co_member)
        VALUES ($1, $2, $3, $4 )
        RETURNING id, room_name, is_group, created_by, bookmarked_by, archived_by, co_member, is_public, created_at, updated_at
        "#,
    )
    .bind(&co_member.full_name)
    .bind(false)
    .bind(created_by)
    .bind(&co_member.id)
    .fetch_one(&state.db)
    .await
    {
        Ok(room) => room,
        Err(e) => {
            error!("ROOM CREATION ERROR!");

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Response {
                    response_message: "Room creation error".into(),
                    response: None,
                    error: Some(format!("Server error: {}", e)),
                }),
            );
        }
    };

    // create admin room member
    match sqlx::query_as::<_, RoomMember>(
        r#"
        INSERT INTO room_members (room_id, user_id, role, joined_at)
        VALUES ($1, $2, $3, $4)
        RETURNING  id, room_id, user_id, role, joined_at, created_at, updated_at
        "#,
    )
    .bind(&room.id)
    .bind(created_by)
    .bind("admin")
    .bind(current_time_millis().to_string())
    .fetch_one(&state.db)
    .await
    {
        Ok(room) => room,
        Err(e) => {
            error!("ROOM MEMBER CREATION ERROR!");

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Response {
                    response_message: "Failed to create admin room member".into(),
                    response: None,
                    error: Some(format!("Room member creation error : {}", e)),
                }),
            );
        }
    };

    // create room co-member
    match sqlx::query_as::<_, RoomMember>(
        r#"
        INSERT INTO room_members (room_id, user_id, role, joined_at)
        VALUES ($1, $2, $3, $4)
        RETURNING id, room_id, user_id, role, joined_at, created_at, updated_at
        "#,
    )
    .bind(room.id)
    .bind(co_member.id)
    .bind("member")
    .bind(current_time_millis().to_string())
    .fetch_one(&state.db)
    .await
    {
        Ok(room) => room,
        Err(e) => {
            error!("ROOM MEMBER CREATION ERROR!");

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Response {
                    response_message: "Failed to create room co_member".into(),
                    response: None,
                    error: Some(format!("Room member creation error : {}", e)),
                }),
            );
        }
    };

    (
        StatusCode::OK,
        Json(Response {
            response_message: "Room created successfully".into(),
            response: Some(room),
            error: None,
        }),
    )
}
