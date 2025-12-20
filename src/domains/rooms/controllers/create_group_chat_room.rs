use std::cmp::PartialEq;
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

#[derive(Debug, Deserialize)]
pub struct CreateRoomPayload {
    // pub room_name: String,
    pub co_members: Vec<i64>,
    pub room_name: String
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserLookUp {
    id: i64,
    full_name: String,
    // email: String,
    // profile_image: Option<String>,
    // access_token: String,
    // refresh_token: String,
    // status: String,
    // last_seen: Option<String>,
    // #[serde(skip_serializing)]
    // password: String,
    // is_admin: bool,
    // is_active: bool,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Room {
    pub id: i64,
    pub room_name: Option<String>,
    pub is_group: bool,
    pub created_by: Option<i64>,
    pub bookmarked_by: Vec<i64>,
    pub archived_by: Vec<i64>,
    pub co_members: Vec<i64>, // for group rooms only
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RoomMember {
    pub id: i64,
    pub room_id: i64,
    pub user_id: i64,
    pub role: String,
    pub joined_at: String,
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

pub async fn create_group(
    State(state): State<AppState>,
    // Query(params): Query<SearchParams>,
    Extension(session): Extension<SessionsMiddlewareOutput>,
    Json(payload): Json<CreateRoomPayload>,
) -> impl IntoResponse {
    println!("Time in milliseconds {:?}", current_time_millis());

    let created_by = session.user.id;

    // for (index, &member) in payload.co_members.iter().enumerate() {
    for member in &payload.co_members {
        match sqlx::query_as::<_, UserLookUp>(
            "SELECT id, full_name FROM users WHERE id = $1",
        )
            .bind(member)
            .fetch_optional(&state.db)
            .await
        {
            Ok(Some(member)) => {
                if created_by == member.id {
                    error!("ROOM CREATION ERROR: SELF IN CO-MEMBERS ARRAY!");

                    return (
                        StatusCode::NOT_FOUND,
                        Json(Response {
                            response_message: "Self in co-members array!".to_string(),
                            error: Some("Room creation error".to_string()),
                            response: None,
                        }),
                    )
                }

                member
            },
            Ok(None) => {
                error!("ROOM CREATION ERROR: AT LEAST ONE CO-MEMBER DATA NOT PROVIDED!");

                return (
                    StatusCode::NOT_FOUND,
                    Json(Response {
                        response_message: format!("Co-member with id: '{}' not found", member),
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
                        response_message: format!("Co-member with id: '{}' not found", member),
                        error: Some(format!("Room creation error: {:?}", e)),
                        response: None,
                    })
                )
            }
        };
    };

    let room = match sqlx::query_as::<_, Room>(
        r#"
        INSERT INTO rooms (room_name, is_group, created_by, co_members)
        VALUES ($1, $2, $3, $4)
        RETURNING id, room_name, is_group, created_by, bookmarked_by, archived_by, co_members
        "#,
    )
    .bind(&payload.room_name)
    .bind(true)
    .bind(created_by)
    .bind(&payload.co_members)
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
        RETURNING  id, room_id, user_id, role, joined_at
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
                    response_message: "Failed to create room member 2".into(),
                    response: None,
                    error: Some(format!("Room member creation error : {}", e)),
                }),
            );
        }
    };

    for member in payload.co_members {
        // create room co-members
        match sqlx::query_as::<_, RoomMember>(
            r#"
        INSERT INTO room_members (room_id, user_id, role, joined_at)
        VALUES ($1, $2, $3, $4)
        RETURNING id, room_id, user_id, role, joined_at
        "#,
        )
            .bind(room.id)
            .bind(member)
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
                        response_message: "Failed to create room member".into(),
                        response: None,
                        error: Some(format!("Room member creation error : {}", e)),
                    }),
                );
            }
        };
    }

    (
        StatusCode::OK,
        Json(Response {
            response_message: "Room created successfully".into(),
            response: Some(room),
            error: None,
        }),
    )
}
