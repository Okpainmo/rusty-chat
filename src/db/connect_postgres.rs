use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

pub async fn connect_pg(database_url: String) -> sqlx::PgPool {
    // println!("Attempting to connect to PostgreSQL database...");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await;

    match pool {
        Ok(p) => {
            // println!("Successfully connected to PostgreSQL database.");
            p
        }
        Err(e) => {
            println!(
                "
                CRITICAL DATABASE CONNECTION ERROR:
                -------------------------------------------------
                Error: {}
                URL: {}
                -------------------------------------------------
                Please verify:
                1. Is Postgres running?
                2. Is the connection URL correct?
                3. Are the credentials valid?
                4. Is the network allowing connection to port 5432?
                -------------------------------------------------
                ",
                e, database_url
            );

            panic!("DATABASE CONNECTION FAILED: {}", e);
        }
    }
}
