use aws_sdk_s3::Client;
use axum::extract::multipart::{Field, MultipartError};
use axum::{
    Router,
    extract::Multipart,
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use serde::Serialize;
use std::env;

#[derive(Clone, Debug)]
pub struct S3AppState {
    pub s3_client: Client,
    pub bucket_name: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserProfile {
    id: i64,
    full_name: String,
    email: String,
    profile_image: Option<String>,
    access_token: String,
    refresh_token: String,
    status: String,
    last_seen: Option<String>,
    #[serde(skip_serializing)]
    password: String,
    is_admin: bool,
    is_active: bool,
}

pub struct UpdateResponse {
    response_message: String,
    response: Option<UserProfile>,
    error: Option<String>,
}

pub async fn upload_file(
    State(state): State<&crate::AppState>,
    field: Field<'_>,
    user_id: &i64,
) -> Result<String, MultipartError> {
    // Your implementation

    // Generate unique S3 key
    let extension = field
        .file_name()
        .expect("Failed to extract object file name!")
        .split('.')
        .last()
        .expect("Failed to get object extension");

    let file_key = format!("profile_image_{}.{}", user_id, extension);

    let aws_region = env::var("AWS_S3_BUCKET_REGION").expect("AWS_S3_BUCKET_REGION must be set");

    // Construct the S3 URL
    // Format: https://{bucket}.s3.{region}.amazonaws.com/{key}
    let file_url = format!(
        "https://{}.s3.{}.amazonaws.com/{}",
        state.s3.bucket_name, aws_region, file_key,
    );

    let content_type = field
        .content_type()
        .map(|ct| ct.to_string())
        .expect("Failed to extract object content type!");

    let data = field.bytes().await?.to_vec();

    // Upload to S3
    state
        .s3
        .s3_client
        .put_object()
        .bucket(&state.s3.bucket_name)
        .key(&file_key)
        .content_type(content_type)
        .body(data.into())
        .send()
        .await
        .expect("S3 File upload failed!");

    // println!("{:?}", res);

    Ok(file_url)
}
