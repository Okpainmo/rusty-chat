use axum::{Router, middleware};
use sqlx::PgPool;
use std::sync::Arc;

use std::net::SocketAddr;

// environmental variables...
use std::env;

// AWS S3
use crate::utils::file_upload_handler::S3AppState;
use aws_config::Region;
use aws_credential_types::Credentials;
use aws_sdk_s3::Client;

// logging init with the tracing crate
use tracing::error;
use tracing::info;
use tracing_subscriber::fmt::time::SystemTime;

// utils import
mod utils;
use crate::utils::load_config::{AppConfig, load_config};
use crate::utils::load_env::load_env;
// db import
mod db;
use db::connect_postgres::connect_pg;

// controllers import
mod domains;
use crate::domains::admin::router::admin_routes;
use crate::domains::auth::router::auth_routes;
use crate::domains::messages::router::messages_routes;
use crate::domains::rooms::router::rooms_routes;
use crate::domains::user::router::user_routes;

mod middlewares;
use crate::middlewares::logging_middleware::logging_middleware;
use crate::middlewares::request_timeout_middleware::timeout_middleware;

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Arc<AppConfig>,
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

    let app_config = load_config();

    let clean_config = match app_config {
        Ok(config) => {
            // println!("Configuration loaded successfully: {}", config.app.name);

            if config.validate().is_err() {
                error!("SERVER START-UP ERROR: FAILED TO LOAD SERVER CONFIGURATIONS!");

                return;
            }

            config
        }
        Err(_e) => {
            error!("SERVER START-UP ERROR: FAILED TO LOAD SERVER CONFIGURATIONS!");
            return;
        }
    };

    let access_key_id = env::var("AWS_ACCESS_KEY").unwrap();
    let secret_access_key = env::var("AWS_SECRET_ACCESS_KEY").unwrap();
    // let aws_url = env::var("AWS_BUCKET_URL").unwrap();
    let aws_region = env::var("AWS_S3_BUCKET_REGION").expect("AWS_S3_BUCKET_REGION must be set");

    // note here that the "None" is in place of a session token
    let s3_credentials = Credentials::from_keys(access_key_id, secret_access_key, None);

    let s3_config = aws_config::from_env()
        // .endpoint_url(aws_url)
        .region(Region::new(aws_region))
        .credentials_provider(s3_credentials)
        .load()
        .await;

    // Initialize AWS S3 config
    // let config = aws_config::load_from_env().await;
    let s3_client = Client::new(&s3_config);

    let bucket_name = std::env::var("AWS_S3_BUCKET_NAME").expect("AWS_S3_BUCKET_NAME must be set");

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
        config: Arc::new(clean_config),
        db: db_pool,
        s3: s3_state,
    };

    // fn verify_config_loading(state: &AppState) {
    //     println!(
    //         "Config loaded successfully: {} is running on {}:{}",
    //         state.config.app.name,
    //         state.config.server.as_ref().unwrap().host,
    //         state.config.server.as_ref().unwrap().port
    //     );
    // }

    // verify_config_loading(&state);

    let app = Router::new()
        .nest("/api/v1/auth", auth_routes(&state))
        .nest("/api/v1/user", user_routes(&state))
        .nest("/api/v1/admin", admin_routes(&state))
        .nest("/api/v1/rooms", rooms_routes(&state))
        .nest("/api/v1/messages", messages_routes(&state))
        .layer(middleware::from_fn(logging_middleware))
        .layer(middleware::from_fn(timeout_middleware))
        .with_state(state);

    // .layer(Extension(db_pool));

    // Server address
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));

    let slice_db_url = format!("{}...", &database_url[0..25]);

    // Start server
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
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

            listener
        }
        Err(e) => {
            error!("SERVER INITIALIZATION ERROR: {}!", e);

            return;
        }
    };

    let server_result = axum::serve(listener, app).await;

    match server_result {
        Ok(_) => {
            info!("Graceful server shutdown!");
        }
        Err(e) => {
            error!("SERVER SHUTDOWN ERROR: {}!", e);
        }
    }
}
