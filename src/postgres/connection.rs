use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions, PgSslMode};
use sqlx::Error;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::OnceCell;
use std::time::Duration;

// Static database pool instance
static DB_POOL: OnceCell<Arc<PgPool>> = OnceCell::const_new();

/// Initialize PostgreSQL database connection pool with custom configuration
pub async fn init_db_pool_custom(pool: PgPool) -> Result<&'static PgPool, Error> {
    // Wrap the connection pool in an Arc and initialize OnceCell
    let pool = Arc::new(pool);
    DB_POOL.get_or_try_init(|| async { Ok(pool.clone()) })
        .await
        .map(|arc| arc.as_ref())
}

/// Initialize PostgreSQL database connection pool using a database URL
pub async fn init_db_pool(database_url: &str) -> Result<&'static PgPool, Error> {
    // Parse the database URL and configure connection options
    let connect_options = PgConnectOptions::from_str(database_url)?
        .ssl_mode(PgSslMode::Disable); // Configure SSL mode as needed

    // Create the connection pool and set parameters
    let pool = PgPoolOptions::new()
        .max_connections(50) // Maximum number of connections
        .min_connections(10) // Minimum number of connections
        .acquire_timeout(Duration::from_secs(5)) // Connection acquire timeout
        .idle_timeout(Duration::from_secs(300)) // Idle connection timeout
        .max_lifetime(Duration::from_secs(1800)) // Maximum connection lifetime
        .test_before_acquire(true) // Test connection validity before acquiring
        .connect_with(connect_options)
        .await?;

    // Warm up the connection pool (optional)
    let _ = pool.acquire().await;

    // Initialize the connection pool with custom configuration
    init_db_pool_custom(pool).await
}

/// Get a reference to the PostgreSQL database connection pool
pub fn get_db_pool() -> &'static PgPool {
    DB_POOL.get().expect("PostgreSQL database pool not initialized")
}