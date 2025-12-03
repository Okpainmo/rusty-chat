use axum::{
    extract::Extension,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::net::SocketAddr;
use dotenvy;
use std::env;

// db import
mod db; // include the db folder
use db::connect_postgres::connect_pg;

// utils import
mod utils;
use utils::hashing_handler::hashing_handler;

// ====== Request Data ======
#[derive(Debug, Deserialize)]
struct RegisterRequest {
    first_name: String,
    last_name: String,
    email: String,
    password: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct UserProfile {
    #[sqlx(rename = "id")]
    user_id: i64,
    full_name: String,
    email: String,
    profile_image_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct Response {
    user_profile: UserProfile,
}

// ====== Response Data ======
#[derive(Debug, Serialize)]
struct RegisterResponse {
    response_message: String,    
    response: Response,
    error: Option<String>
}

#[tokio::main]
async fn main() {
    load_env();

    
    // println!("Environment: {}", env);
    // println!("Server running on port {}", port);
    // Build router
    
    // let port = env::var("PORT").unwrap_or("8000".to_string());
    let environment = env::var("DEPLOY_ENV").unwrap_or("development".to_string());
    let user = env::var("POSTGRES_USER").unwrap();
    let pass = env::var("POSTGRES_PASSWORD").unwrap();
    let host = env::var("POSTGRES_HOST").unwrap();
    let db_port = env::var("POSTGRES_PORT").unwrap();
    let db = env::var("POSTGRES_DB").unwrap();

    let database_url = format!("postgres://{}:{}@{}:{}/{}", user, pass, host, db_port, db);

    let db_pool = connect_pg(database_url).await;

    let app = Router::new()
        .route("/api/v1/auth/register", post(register_handler))
        .layer(Extension(db_pool));

    // Server address
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));

    println!(
        "
        .................................................
        Connected to DB Host: {}
        Environment: {}
        Status: DB connected successfully
        .................................................

        Server running on http://{}
        ",
        host, environment, addr
    );

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn load_env() {
    dotenvy::dotenv().ok(); // loads .env first

    let env = env::var("DEPLOY_ENV").unwrap_or("development".into());
    let filename = format!(".env.{}", env);

    dotenvy::from_filename(&filename).ok();
    // println!("Loaded config: {}", filename);
}

// ====== Handler for POST /register ======
async fn register_handler(
    Extension(db_pool): Extension<PgPool>,
    Json(payload): Json<RegisterRequest>
) -> Json<RegisterResponse> {
    // Hash the password
    let hashed_password = match hashing_handler(payload.password.as_str()).await {
        Ok(hash) => hash,
        Err(e) => {
            return Json(RegisterResponse {
                response_message: "Failed to hash password".to_string(),
                response: Response {
                    user_profile: UserProfile {
                        user_id: 0i64,
                        full_name: String::new(),
                        email: String::new(),
                        profile_image_url: None,
                    },
                },
                error: Some(format!("Password hashing error: {}", e)),
            });
        }
    };

    // Check if email already exists
    let email_exists: Option<i64> = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users WHERE email = $1"
    )
    .bind(&payload.email)
    .fetch_optional(&db_pool)
    .await
    .unwrap_or(None)
    .flatten();

    if let Some(count) = email_exists {
        if count > 0 {
            return Json(RegisterResponse {
                response_message: "Registration failed".to_string(),
                response: Response {
                    user_profile: UserProfile {
                        user_id: 0i64,
                        full_name: String::new(),
                        email: String::new(),
                        profile_image_url: None,
                    },
                },
                error: Some("Email already exists".to_string()),
            });
        }
    }

    // Insert user into database
    let full_name = format!("{} {}", payload.first_name, payload.last_name);
    
    let result = sqlx::query_as::<_, UserProfile>(
        r#"
            INSERT INTO users (first_name, last_name, email, password, full_name)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, full_name, email, profile_image_url
        "#
    )
    .bind(&payload.first_name)
    .bind(&payload.last_name)
    .bind(&payload.email)
    .bind(&hashed_password)
    .bind(&full_name)
    .fetch_one(&db_pool)
    .await;

    match result {
        Ok(new_user) => Json(RegisterResponse {
            response_message: format!("User with email '{}' registered successfully!", payload.email),
            response: Response {
                user_profile: new_user,
            },
            error: None,
        }),
        Err(e) => {
            // Handle unique constraint violations or other DB errors
            let error_msg = if e.to_string().contains("unique") || e.to_string().contains("duplicate") {
                "Email already exists".to_string()
            } else {
                format!("Database error: {}", e)
            };

            Json(RegisterResponse {
                response_message: "Failed to register user".to_string(),
                response: Response {
                    user_profile: UserProfile {
                        user_id: 0i64,
                        full_name: String::new(),
                        email: String::new(),
                        profile_image_url: None,
                    },
                },
                error: Some(error_msg),
            })
        }
    }
}
