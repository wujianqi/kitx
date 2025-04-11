use crate::common::error::OperationError;

use sqlx::{Pool, Sqlite};
use sqlx::{pool::PoolOptions, Error, SqlitePool};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::OnceCell;
use std::time::Duration;

use crate::utils::db;

// Global static variable to store the database connection pool
static DB_POOL: OnceCell<Arc<SqlitePool>> = OnceCell::const_new();

// Initialize the connection pool with a custom pool
pub async fn init_db_pool_custom<'a>(pool: Pool<Sqlite>) -> Result<&'a SqlitePool, Error> {
    // Create the connection pool
    let pool = Arc::new(pool);

    // Force evaluation of OnceCell to ensure the connection pool is initialized
    DB_POOL.get_or_try_init(|| async { Ok(pool) }).await
        .map(|arc| arc.as_ref())
}

/// Initializes the database connection pool with the database URL and enables WAL mode.
pub async fn init_db_pool(database_url: &str) -> Result<&SqlitePool, Error> {
    let (maxc, minc, _) = db::connect_limits(None);

    let connect_options = SqliteConnectOptions::from_str(database_url)
        .map_err(|e| OperationError::db(format!("Failed to parse SQLite connection URL: {}", e)))?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal);

    let pool = PoolOptions::new()
        .max_connections(maxc)
        .min_connections(minc)
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(30))
        .connect_with(connect_options)
        .await
        .map_err(|e| OperationError::db(format!("Failed to initialize SQLite connection pool: {}", e)))?;

    init_db_pool_custom(pool).await
}

/// Gets a reference to the database connection pool.
pub fn get_db_pool() -> Result<Arc<SqlitePool>, Error> {
    DB_POOL.get()
        .cloned() // Clone the Arc to return a new reference
        .ok_or_else(|| OperationError::db("Database pool not initialized".to_string()))
}