use std::time::Duration;

use sqlx::{postgres::PgPoolOptions, PgPool};

use tracing::debug;

pub async fn connect(url: &str) -> PgPool {
    debug!("Connecting to database");

    PgPoolOptions::new()
        .max_connections(50)
        .acquire_timeout(Duration::from_secs(3))
        .connect(url)
        .await
        .expect("Should be able to connect to the database")
}
