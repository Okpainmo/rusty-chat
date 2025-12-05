use axum::{
    extract::Extension,
    routing::post,
    Router,
};

use std::net::SocketAddr;
use dotenvy;
use std::env;

// logging init with the tracing crate
use tracing_subscriber::fmt::time::SystemTime;

// utils import
pub mod utils;
// db import
mod db; // include the db folder
use db::connect_postgres::connect_pg;

// controllers import
mod controllers;
use controllers::register_user::register_user;
use controllers::login_user::login_user;



fn load_env() {
    dotenvy::dotenv().ok();

    let env = env::var("DEPLOY_ENV").unwrap_or("development".into());
    let filename = format!(".env.{}", env);

    dotenvy::from_filename(&filename).ok();
    // println!("Loaded config: {}", filename);
}

// IntoResponse setup for custom status code usage
// impl<T: Serialize> IntoResponse for (StatusCode, ApiResponse<T>) {
//     fn into_response(self) -> Response {
//         let (status, body) = self;
//         (status, Json(body)).into_response()
//     }
// }

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
    // info!("DB = {:?}", std::env::var("DATABASE_URL"));

    
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

    let db_pool = connect_pg(database_url.clone()).await;

    let app = Router::new()
        .route("/api/v1/auth/register", post(register_user))
        .route("/api/v1/auth/log-in", post(login_user))
        .layer(Extension(db_pool));

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
        slice_db_url,
        environment,
        addr
    );

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}