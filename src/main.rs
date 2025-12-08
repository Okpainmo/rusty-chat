use axum::{
    extract::Extension,
    routing::post,
    Router,
};

use std::net::SocketAddr;

// environmental variables... 
use dotenvy;
use std::env;
use tracing::info;
// logging init with the tracing crate
use tracing_subscriber::fmt::time::SystemTime;

// utils import
pub mod utils;
// db import
mod db; // include the db folder
use db::connect_postgres::connect_pg;
use crate::utils::load_env::load_env;

// controllers import
mod domains;
mod middlewares;

use crate::domains::auth::router::auth_routes;
use crate::domains::user::router::user_routes;

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
    // info!("DB = { }", std::env::var("DATABASE_URL").unwrap().to_string());

    
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
        .nest("/api/v1", auth_routes())
        .nest("/api/v1", user_routes())
        .layer(Extension(db_pool));
    // .nest("/users", user_routes());


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