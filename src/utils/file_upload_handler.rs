// use aws_sdk_s3::model::ObjectCannedAcl;
// use aws_sdk_s3::{Client as S3Client, types::ByteStream};
// use axum::{Json, extract::Multipart, http::StatusCode, response::IntoResponse};
// use std::sync::Arc;
//
// #[derive(serde::Serialize)]
// struct UploadResponse {
//     file_url: String,
// }
//
// pub async fn upload_file(mut multipart: Multipart, s3_client: Arc<S3Client>) -> impl IntoResponse {
//     // Get the first field from multipart
//     while let Some(field) = multipart.next_field().await.unwrap() {
//         let name = field.name().unwrap_or("file");
//         let file_name = field.file_name().unwrap_or("upload.bin");
//         let data = field.bytes().await.unwrap();
//
//         // Generate unique key for S3
//         let key = format!("uploads/{}_{}", Uuid::new_v4(), file_name);
//
//         // Upload to S3
//         let bucket_name = std::env::var("AWS_BUCKET_NAME").unwrap();
//         let result = s3_client
//             .put_object()
//             .bucket(bucket_name)
//             .key(&key)
//             .body(ByteStream::from(data.to_vec()))
//             .acl(ObjectCannedAcl::PublicRead) // optional
//             .send()
//             .await;
//
//         match result {
//             Ok(_) => {
//                 let file_url = format!(
//                     "https://{}.s3.amazonaws.com/{}",
//                     std::env::var("AWS_BUCKET_NAME").unwrap(),
//                     key
//                 );
//                 return (StatusCode::OK, Json(UploadResponse { file_url }));
//             }
//             Err(e) => {
//                 return (
//                     StatusCode::INTERNAL_SERVER_ERROR,
//                     Json(serde_json::json!({"error": format!("Upload failed: {}", e)})),
//                 );
//             }
//         }
//     }
//
//     (
//         StatusCode::BAD_REQUEST,
//         Json(serde_json::json!({"error": "No file provided"})),
//     )
// }
