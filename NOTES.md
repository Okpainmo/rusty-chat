# Notes

## Making a clean project refresh.

```shell
cargo clean
cargo update
cargo build
```

## Working with the AWS S3 SDK, you'll need to install CMAKE, NASM and Visual Studio

- CMAKE: https://cmake.org/download/
- NASM: https://www.nasm.us
- VISUAL STUDIO: https://visualstudio.microsoft.com/downloads/

> After installation, ensure to add both to system PATH. Then restart system, and ensure that the project on the system is not too long, as that might trigger the windows "path too long" error.
 
## Rust Multipart form handling.

```rust
pub async fn update_user(
    cookies: Cookies,
    State(state): State<AppState>,
    Extension(session): Extension<SessionsMiddlewareOutput>,
    Path(user_id): Path<i64>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut payload: Option<UpdateUserPayload> = None;
    let mut file: Option<Field> = None;

    // Process all multipart fields
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        
        match name.as_str() {
            "file" => {
                file = Some(field);
            }
            "data" => {
                // Parse JSON data from a multipart field
                let data = field.bytes().await.unwrap();
                payload = serde_json::from_slice(&data).ok();
            }
            _ => {}
        }
    }

    let Some(payload) = payload else {
        return (
            StatusCode::BAD_REQUEST,
            Json(UpdateResponse {
                response_message: "Missing payload data".to_string(),
                response: None,
                error: Some("No data field provided".to_string()),
            }),
        ).into_response();
    };

    // Now process with optional file
    if let Some(file) = file {
        // Handle file upload
    }
    
    // Continue with your update logic...
}
```
