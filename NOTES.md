# Notes

## Important feature to add.

- Feature name: "Spaces" - a version of self messaging in Whatsapp. But more like notes spaces that you 
create to jot down valuable stuff. Advantage is you can create and customize multiple spaces for yourself -
for taking notes on different topics. Also consider a way to share external read-only access to one's spaces. 
The read-only access is actually not necessary since the goal is to provide kind of a self messaging version of
what is on Whatsapp.

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

> **To finally disable the Windows path-too-long error, simply enable the allow-long-path features in Windows**
 
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

## Docker PG DB start command.

```shell
# update the start command to suit your setup, and start databases for all the 3 environments using docker.

docker run -d --name rusty-chat__dev_db -p 5433:5432 -e POSTGRES_USER=okpainmo -e POSTGRES_PASSWORD=supersecret -e POSTGRES_DB=rusty_chat_db_dev postgres
```
