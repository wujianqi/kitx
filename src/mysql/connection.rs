use sqlx::{Pool, MySql};
use sqlx::{pool::PoolOptions, Error, MySqlPool};
use sqlx::mysql::{MySqlConnectOptions, MySqlSslMode};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::OnceCell;
use std::time::Duration;

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
    println!("get_db_pool");
    // Configure MySQL connection options
    let connect_options = MySqlConnectOptions::from_str(database_url)? // Parse the database URL
        .ssl_mode(MySqlSslMode::Disabled); // Configure SSL mode as needed

    // Create the connection pool with specified options
    let pool = PoolOptions::new()
        .max_connections(50) // Adjust the maximum number of connections based on load
        .min_connections(10) // Pre-establish minimum connections to reduce initial request latency
        .acquire_timeout(Duration::from_secs(5)) // Set the timeout for acquiring a connection
        .idle_timeout(Duration::from_secs(300)) // Set the timeout for idle connections
        .max_lifetime(Duration::from_secs(1800)) // Set the maximum lifetime of a connection
        .test_before_acquire(true) // Test the connection before acquiring it
        .connect_with(connect_options)
        .await?;

    // Warm up the connection pool by pre-establishing the minimum number of connections
    let _ = pool.acquire().await;

    // Initialize the connection pool with the created pool
    init_db_pool_custom(pool).await
}

/// Gets a reference to the database connection pool.
pub fn get_db_pool() -> &'static MySqlPool {
    DB_POOL.get().expect("Database pool not initialized")
}
