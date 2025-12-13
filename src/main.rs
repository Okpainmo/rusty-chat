use axum::{Router, extract::Extension, middleware, routing::post};

use std::net::SocketAddr;

// environmental variables...
use dotenvy;
use std::env;

// AWS S3
use aws_sdk_s3::Client;
use crate::utils::file_upload_handler::S3AppState;

use tracing::info;
// logging init with the tracing crate
use tracing_subscriber::fmt::time::SystemTime;

// utils import
mod utils;
// db import
mod db;
use sqlx::PgPool;
use crate::utils::load_env::load_env;
use db::connect_postgres::connect_pg;

// controllers import
mod domains;
use crate::domains::auth::router::auth_routes;
use crate::domains::user::router::user_routes;
use crate::domains::admin::router::admin_routes;


mod middlewares;
use crate::middlewares::logging_middleware::logging_middleware;
use crate::middlewares::request_timeout_middleware::timeout_middleware;

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: PgPool,
    pub s3: S3AppState,
}

fn initialize_logging() {
    tracing_subscriber::fmt()
        .json()
        .with_timer(SystemTime)
        // .with_thread_ids(true)
        .with_level(true)
        .init();
}

#[tokio::main]
async fn main() {
    load_env();
    initialize_logging();

    // Initialize AWS S3 config
    let config = aws_config::load_from_env().await;
    let s3_client = Client::new(&config);

    let bucket_name = std::env::var("AWS_S3_BUCKET_NAME")
        .expect("S3_BUCKET_NAME must be set");

    let s3_state = S3AppState {
        s3_client,
        bucket_name,
    };

    // let port = env::var("PORT").unwrap_or("8000".to_string());
    let environment = env::var("DEPLOY_ENV").unwrap_or("development".to_string());
    let user = env::var("POSTGRES_USER").unwrap();
    let pass = env::var("POSTGRES_PASSWORD").unwrap();
    let host = env::var("POSTGRES_HOST").unwrap();
    let db_port = env::var("POSTGRES_PORT").unwrap();
    let db = env::var("POSTGRES_DB").unwrap();

    let database_url = format!("postgres://{}:{}@{}:{}/{}", user, pass, host, db_port, db);

    let db_pool = connect_pg(database_url.clone()).await;

    let state = AppState {
        db: db_pool,
        s3: s3_state
    };

    let app = Router::new()
        .nest("/api/v1", auth_routes(&state))
        .nest("/api/v1", user_routes(&state))
        .nest("/api/v1", admin_routes(&state))
        .layer(middleware::from_fn(logging_middleware))
        .layer(middleware::from_fn(timeout_middleware))
        .with_state(state);

    // .layer(Extension(db_pool));

    // Server address
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));

    let slice_db_url = format!("{}...", &database_url[0..40]);

    print!(
        "
        .................................................
        Connected to DB: {}
        Environment: {}
        Status: DB connected successfully
        .................................................

        Server running on http://{}
        ",
        slice_db_url, environment, addr
    );

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
