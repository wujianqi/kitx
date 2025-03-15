use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions, PgSslMode};
use sqlx::Error;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::OnceCell;
use std::time::Duration;

use crate::common::util;

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
    let (maxc, minc, warmupc) = util::db_connect_limits(Some(20));

    // Parse the database URL and configure connection options
    let connect_options = PgConnectOptions::from_str(database_url)?
        .ssl_mode(PgSslMode::Disable); // Configure SSL mode as needed

    // Create the connection pool and set parameters
    let pool = PgPoolOptions::new()
        .max_connections(maxc)
        .min_connections(minc)
        .acquire_timeout(Duration::from_secs(3)) // Connection acquire timeout
        .idle_timeout(Duration::from_secs(60)) // Idle connection timeout
        .max_lifetime(Duration::from_secs(600)) // Maximum connection lifetime
        .test_before_acquire(true) // Test connection validity before acquiring
        .connect_with(connect_options)
        .await?;

    // Warm up the connection pool (optional)
    let _ = warmup_connect(&pool, warmupc).await;

    // Initialize the connection pool with custom configuration
    init_db_pool_custom(pool).await
}

async fn warmup_connect(pool: &PgPool, warmup_num: u32) -> Result<(), Error> {
    for _ in 0..warmup_num {
        let conn = pool.acquire().await?;
        drop(conn);
    }
    Ok(())
}

/// Get a reference to the PostgreSQL database connection pool
pub fn get_db_pool() -> &'static PgPool {
    DB_POOL.get().expect("PostgreSQL database pool not initialized")
}