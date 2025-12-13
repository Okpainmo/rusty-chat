// use axum::{
//     extract::{Multipart, State},
//     http::StatusCode,
//     response::Json,
//     routing::{get, post},
//     Router,
// };
// use aws_sdk_s3::Client;
// use serde::Serialize;
// use uuid::Uuid;
//
// // App state
// #[derive(Clone)]
// struct AppState {
//     s3_client: Client,
//     bucket_name: String,
// }
//
// // Response types
// #[derive(Serialize)]
// struct UploadResponse {
//     file_key: String,
//     message: String,
// }
//
// #[derive(Serialize)]
// struct ErrorResponse {
//     error: String,
// }
//
// #[tokio::main]
// async fn main() {
//     // Initialize AWS config
//     let config = aws_config::load_from_env().await;
//     let s3_client = Client::new(&config);
//
//     let bucket_name = std::env::var("S3_BUCKET_NAME")
//         .expect("S3_BUCKET_NAME must be set");
//
//     let state = AppState {
//         s3_client,
//         bucket_name,
//     };
//
//     // Routes
//     let app = Router::new()
//         .route("/", get(|| async { "Server running" }))
//         .route("/upload", post(upload_file))
//         .with_state(state);
//
//     // Start server
//     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
//         .await
//         .unwrap();
//
//     println!("Server running on http://localhost:3000");
//     axum::serve(listener, app).await.unwrap();
// }
//
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
//     let file_key = format!("{}-{}", Uuid::new_v4(), filename);
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