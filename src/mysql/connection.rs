use sqlx::{Pool, MySql};
use sqlx::{pool::PoolOptions, Error, MySqlPool};
use sqlx::mysql::{MySqlConnectOptions, MySqlSslMode};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::OnceCell;
use std::time::Duration;

use crate::common::util;

static DB_POOL: OnceCell<Arc<MySqlPool>> = OnceCell::const_new();

/// Initializes the database connection pool with custom settings.
pub async fn init_db_pool_custom(pool: Pool<MySql>) -> Result<&'static MySqlPool, Error> {
    // Create the connection pool
    let pool = Arc::new(pool);

    // Force initialization of OnceCell to ensure the connection pool is initialized
    DB_POOL.get_or_try_init(|| async { Ok(pool.clone()) }).await
        .map(|arc| arc.as_ref())
}

/// Initializes the database connection pool with a database URL.
pub async fn init_db_pool(database_url: &str) -> Result<&'static MySqlPool, Error> {
    let (maxc, minc, warmupc) = util::db_connect_limits(Some(20));

    // Configure MySQL connection options
    let connect_options = MySqlConnectOptions::from_str(database_url)? // Parse the database URL
        .ssl_mode(MySqlSslMode::Disabled); // Configure SSL mode as needed

    // Create the connection pool with specified options
    let pool = PoolOptions::new()
        .max_connections(maxc) // Adjust the maximum number of connections based on load
        .min_connections(minc) // Pre-establish minimum connections to reduce initial request latency
        .acquire_timeout(Duration::from_secs(3)) // Set the timeout for acquiring a connection
        .idle_timeout(Duration::from_secs(60)) // Set the timeout for idle connections
        .max_lifetime(Duration::from_secs(600)) // Set the maximum lifetime of a connection
        .test_before_acquire(true) // Test the connection before acquiring it
        .connect_with(connect_options)
        .await?;

    // Warm up the connection pool by pre-establishing the minimum number of connections
    let _ = warmup_connect(&pool, warmupc).await;

    // Initialize the connection pool with the created pool
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
pub fn get_db_pool() -> &'static MySqlPool {
    DB_POOL.get().expect("Database pool not initialized")
}
