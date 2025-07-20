use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions, PgSslMode};
use sqlx::Error;
use std::cmp::max;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::OnceCell;
use std::time::Duration;

use crate::common::error::QueryError;

// Static database pool instance
static DB_POOL: OnceCell<Arc<PgPool>> = OnceCell::const_new();

fn connect_limits() -> (u32, u32, u32) {
    let percentage = Some(20);
    let num_cpus = num_cpus::get() as u32;
    let max_connections = max(20, num_cpus * 3);
    let min_connections = max(2, num_cpus / 2);
    let warmup_connections = percentage.map_or(0, |perc| {
        (max_connections as f32 * (perc as f32 / 100.0)).ceil() as u32
    });
    
    (max_connections, min_connections, warmup_connections)
}

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
    let (maxc, minc, warmupc) = connect_limits();

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