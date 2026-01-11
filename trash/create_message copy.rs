use crate::AppState;
use crate::middlewares::auth_sessions_middleware::SessionsMiddlewareOutput;
use crate::utils::current_time_in_milliseconds;
use crate::utils::file_upload_handler::{UploadType, upload_file, streaming_upload};
use axum::{
    Json,
    extract::{Extension, Multipart, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::NaiveDateTime;
use serde::Serialize;
use tracing::error;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Message {
    pub id: i64,
    pub room_id: i64,
    pub sender_id: Option<i64>,
    #[sqlx(rename = "type")]
    pub message_type: String,
    pub text_content: Option<String>,
    pub attachment_1: Option<String>,
    pub attachment_2: Option<String>,
    pub attachment_3: Option<String>,
    pub attachment_4: Option<String>,
    pub status: String,
    pub sent_at: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct CreateMessageResponse {
    pub response_message: String,
    pub response: Option<Message>,
    pub error: Option<String>,
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

pub async fn create_message(
    State(state): State<AppState>,
    Extension(session): Extension<SessionsMiddlewareOutput>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut room_id: Option<i64> = None;
    let mut message_type: Option<String> = None;
    let mut text_content: Option<String> = None;
    let mut attachment_1: Option<String> = None;
    let mut attachment_2: Option<String> = None;
    let mut attachment_3: Option<String> = None;
    let mut attachment_4: Option<String> = None;

    // Generate batch ID for this message's attachments
    let batch_id = Uuid::new_v4().to_string();

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "room_id" => {
                let val = field.text().await;
                let text = match val {
                    Ok(id) => id,
                    Err(_) => {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(CreateMessageResponse {
                                response_message: "Room ID is required".to_string(),
                                response: None,
                                error: Some("Missing field: room_id".to_string()),
                            }),
                        );
                    }
                };

                let parsed = text.parse::<i64>();
                if let Ok(id) = parsed {
                    room_id = Some(id);
                } else {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(CreateMessageResponse {
                            response_message: "Room ID is required".to_string(),
                            response: None,
                            error: Some("Invalid room_id".to_string()),
                        }),
                    );
                }
            }
            "type" => {
                if let Ok(val) = field.text().await {
                    message_type = Some(val);
                }
            }
            "text_content" => {
                text_content = field.text().await.ok();
            }
            "attachment_1" => {
                if let Ok(url) = streaming_upload(
                    State(&state),
                    field,
                    &batch_id,
                    UploadType::MessageAttachment_1,
                )
                .await
                {
                    attachment_1 = Some(url);
                }
            }
            "attachment_2" => {
                if let Ok(url) = streaming_upload(
                    State(&state),
                    field,
                    &batch_id,
                    UploadType::MessageAttachment_2,
                )
                .await
                {
                    attachment_2 = Some(url);
                }
            }
            "attachment_3" => {
                if let Ok(url) = streaming_upload(
                    State(&state),
                    field,
                    &batch_id,
                    UploadType::MessageAttachment_3,
                )
                .await
                {
                    attachment_3 = Some(url);
                }
            }
            "attachment_4" => {
                if let Ok(url) = streaming_upload(
                    State(&state),
                    field,
                    &batch_id,
                    UploadType::MessageAttachment_4,
                )
                .await
                {
                    attachment_4 = Some(url);
                }
            }
            _ => {}
        }
    }

    // Validate required fields
    let room_id = match room_id {
        Some(id) => id,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CreateMessageResponse {
                    response_message: "Room ID is required".to_string(),
                    response: None,
                    error: Some("Missing field: room_id".to_string()),
                }),
            );
        }
    };

    // Verify user is a member of the room
    let membership = sqlx::query_as::<_, RoomMember>(
        r#"
        SELECT *
        FROM room_members
        WHERE room_id = $1 AND user_id = $2
        "#,
    )
    .bind(&room_id)
    .bind(&session.user.id)
    .fetch_optional(&state.db)
    .await;

    match membership {
        Ok(Some(_)) => (),
        Ok(None) => {
            return (
                StatusCode::FORBIDDEN,
                Json(CreateMessageResponse {
                    response_message: "You are not a member of this room".to_string(),
                    response: None,
                    error: Some("Forbidden".to_string()),
                }),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CreateMessageResponse {
                    response_message: "Failed to verify room membership".to_string(),
                    response: None,
                    error: Some(e.to_string()),
                }),
            );
        }
    }

    let sent_at = current_time_in_milliseconds::current_time_millis();

    // Create message with attachments
    let res = sqlx::query_as::<_, Message>(
        r#"
        INSERT INTO messages (room_id, sender_id, type, text_content, attachment_1, attachment_2, attachment_3, attachment_4, status, sent_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING *
        "#
    )
    .bind(&room_id)
    .bind(&session.user.id)
    .bind(&message_type)
    .bind(&text_content)
    .bind(&attachment_1)
    .bind(&attachment_2)
    .bind(&attachment_3)
    .bind(&attachment_4)
    .bind("sent".to_string())
    .bind(sent_at.to_string())
    .fetch_one(&state.db)
    .await;

    match res {
        Ok(msg) => {
            // Create "sent" status receipt for the sender
            let receipt_res = sqlx::query(
                r#"
                INSERT INTO message_status_receipts (message_id, user_id, room_id, status)
                VALUES ($1, $2, $3, 'sent')
                ON CONFLICT (message_id, user_id) DO NOTHING
                "#
            )
            .bind(msg.id)
            .bind(session.user.id)
            .bind(room_id)
            .execute(&state.db)
            .await;

            if let Err(e) = receipt_res {
                error!("MESSAGE CREATED SUCCESSFULLY, BUT FAILED TO CREATE MESSAGE STATUS RECEIPT: {}", e);
                
                return (
                    StatusCode::CREATED,
                    Json(CreateMessageResponse {
                        response_message: "Message created successfully but failed to create message status receipt".to_string(),
                        response: Some(msg),
                        error: None,
                    }),
                );
            }
    
            (
                StatusCode::CREATED,
                Json(CreateMessageResponse {
                    response_message: "Message created successfully".to_string(),
                    response: Some(msg),
                    error: None,
                }),
            )
        }
        Err(e) => {
            error!("FAILED TO CREATE MESSAGE: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CreateMessageResponse {
                    response_message: "Failed to create message".to_string(),
                    response: None,
                    error: Some(e.to_string()),
                }),
            )
        }
    }
}