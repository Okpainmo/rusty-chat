use axum::{
    extract::State,
    // extract:: Multipart,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use aws_sdk_s3::Client;
use serde::Serialize;

// App state
#[derive(Clone)]
struct AppState {
    s3_client: Client,
    bucket_name: String,
}

// Response types
#[derive(Serialize)]
struct UploadResponse {
    file_key: String,
    message: String,
}

#[derive(Clone, Debug)]
pub struct S3AppState {
    pub s3_client: Client,
    pub bucket_name: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}


// async fn upload_file(
//     State(state): State<AppState>,
//     mut multipart: Multipart,
// ) -> Result<Json<UploadResponse>, (StatusCode, Json<ErrorResponse>)> {
//
//     // Get the first field (the file)
//     let field = multipart
//         .next_field()
//         .await
//         .map_err(|e| {
//             (
//                 StatusCode::BAD_REQUEST,
//                 Json(ErrorResponse {
//                     error: format!("Failed to read field: {}", e),
//                 }),
//             )
//         })?
//         .ok_or_else(|| {
//             (
//                 StatusCode::BAD_REQUEST,
//                 Json(ErrorResponse {
//                     error: "No file provided".to_string(),
//                 }),
//             )
//         })?;
//
//     // Get filename
//     let filename = field
//         .file_name()
//         .unwrap_or("unnamed")
//         .to_string();
//
//     // Read file bytes
//     let data = field.bytes().await.map_err(|e| {
//         (
//             StatusCode::INTERNAL_SERVER_ERROR,
//             Json(ErrorResponse {
//                 error: format!("Failed to read file: {}", e),
//             }),
//         )
//     })?;
//
//     // Generate unique S3 key
//     let file_key = "my_key";
//
//     // Upload to S3
//     state
//         .s3_client
//         .put_object()
//         .bucket(&state.bucket_name)
//         .key(&file_key)
//         .body(data.into())
//         .send()
//         .await
//         .map_err(|e| {
//             (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 Json(ErrorResponse {
//                     error: format!("S3 upload failed: {}", e),
//                 }),
//             )
//         })?;
//
//     Ok(Json(UploadResponse {
//         file_key: file_key.clone(),
//         message: format!("File uploaded successfully: {}", file_key),
//     }))
// }