use crate::common::error::OperationError;

use sqlx::{Pool, MySql};
use sqlx::{pool::PoolOptions, Error, MySqlPool};
use sqlx::mysql::{MySqlConnectOptions, MySqlSslMode};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::OnceCell;
use std::time::Duration;

use crate::utils::db;

static DB_POOL: OnceCell<Arc<MySqlPool>> = OnceCell::const_new();

/// Initializes the database connection pool with custom settings.
pub async fn init_db_pool_custom(pool: Pool<MySql>) -> Result<&'static MySqlPool, Error> {
    // Create the connection pool
    let pool = Arc::new(pool);

    // Force initialization of OnceCell to ensure the connection pool is initialized
    DB_POOL.get_or_try_init(|| async { Ok(pool) }).await
        .map(|arc| arc.as_ref())
}

/// Initializes the database connection pool with a database URL.
pub async fn init_db_pool(database_url: &str) -> Result<&'static MySqlPool, Error> {
    let (maxc, minc, warmupc) = db::connect_limits(Some(20));

    let connect_options = MySqlConnectOptions::from_str(database_url)
        .map_err(|e| OperationError::db(format!("Failed to parse MySQL connection URL: {}", e)))?
        .ssl_mode(MySqlSslMode::Disabled);

    let pool = PoolOptions::new()
        .max_connections(maxc)
        .min_connections(minc)
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(60))
        .max_lifetime(Duration::from_secs(600))
        .test_before_acquire(true)
        .connect_with(connect_options)
        .await
        .map_err(|e| OperationError::db(format!("Failed to initialize MySQL connection pool: {}", e)))?;

    let _ = warmup_connect(&pool, warmupc).await;

    init_db_pool_custom(pool).await
}

async fn warmup_connect(pool: &MySqlPool, warmup_num: u32) -> Result<(), Error> {
    for _ in 0..warmup_num {
        let conn = pool.acquire().await?;
        drop(conn);
    }
    Ok(())
}

/// Gets a reference to the database connection pool.
pub fn get_db_pool() -> Result<Arc<MySqlPool>, Error> {
    DB_POOL.get()
        .cloned()
        .ok_or_else(||{
        OperationError::db("Database pool not initialized".to_string())
    })
}
