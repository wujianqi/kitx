use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions, PgSslMode};
use sqlx::Error;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::OnceCell;
use std::time::Duration;

use crate::common::error::QueryError;
use crate::utils::db;

// Static database pool instance
static DB_POOL: OnceCell<Arc<PgPool>> = OnceCell::const_new();

/// Initialize PostgreSQL database connection pool with custom configuration
pub async fn setup_db_pool(pool: PgPool) -> Result<&'static PgPool, Error> {
    // Wrap the connection pool in an Arc and initialize OnceCell
    let pool = Arc::new(pool);
    DB_POOL.get_or_try_init(|| async { Ok(pool) })
        .await
        .map(|arc| arc.as_ref())
}

/// Initialize PostgreSQL database connection pool using a database URL
pub async fn create_db_pool(database_url: &str) -> Result<&'static PgPool, Error> {
    let (maxc, minc, warmupc) = db::connect_limits(Some(20));

    let connect_options = PgConnectOptions::from_str(database_url)
        .map_err(|e| Error::from(e))?
        .ssl_mode(PgSslMode::Disable);

    let pool = PgPoolOptions::new()
        .max_connections(maxc)
        .min_connections(minc)
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(60))
        .max_lifetime(Duration::from_secs(600))
        .test_before_acquire(true)
        .connect_with(connect_options)
        .await
        .map_err(|e| Error::from(e))?;

    let _ = warmup_connect(&pool, warmupc).await;

    setup_db_pool(pool).await
}

async fn warmup_connect(pool: &PgPool, warmup_num: u32) -> Result<(), Error> {
    for _ in 0..warmup_num {
        let conn = pool.acquire().await?;
        drop(conn);
    }
    Ok(())
}

/// Get a reference to the PostgreSQL database connection pool
pub fn get_db_pool() -> Result<Arc<PgPool>, Error> {
    DB_POOL.get()
        .cloned()
        .ok_or_else(|| QueryError::DBPoolNotInitialized.into())
}