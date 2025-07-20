use crate::common::error::QueryError;

use sqlx::{Pool, MySql};
use sqlx::{pool::PoolOptions, Error, MySqlPool};
use sqlx::mysql::{MySqlConnectOptions, MySqlSslMode};
use std::cmp::max;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::OnceCell;
use std::time::Duration;

static DB_POOL: OnceCell<Arc<MySqlPool>> = OnceCell::const_new();

fn connect_limits() -> (u32, u32, u32) {
    let percentage = Some(20);
    let num_cpus = num_cpus::get() as u32;
    let max_connections = max(10, num_cpus * 2);
    let min_connections = max(2, num_cpus / 2);
    let warmup_connections = percentage.map_or(0, |perc| {
        (max_connections as f32 * (perc as f32 / 100.0)).ceil() as u32
    });
    
    (max_connections, min_connections, warmup_connections)
}

/// Initializes the database connection pool with custom settings.
pub async fn setup_db_pool(pool: Pool<MySql>) -> Result<&'static MySqlPool, Error> {
    // Create the connection pool
    let pool = Arc::new(pool);

    // Force initialization of OnceCell to ensure the connection pool is initialized
    DB_POOL.get_or_try_init(|| async { Ok(pool) }).await
        .map(|arc| arc.as_ref())
}

/// Initializes the database connection pool with a database URL.
pub async fn create_db_pool(database_url: &str) -> Result<&'static MySqlPool, Error> {
    let (maxc, minc, warmupc) = connect_limits();

    let connect_options = MySqlConnectOptions::from_str(database_url)
        .map_err(|e| Error::from(e))?
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
        .map_err(|e| Error::from(e))?;

    let _ = warmup_connect(&pool, warmupc).await;

    setup_db_pool(pool).await
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
        .ok_or_else(||QueryError::DBPoolNotInitialized.into())
}
