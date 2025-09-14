//! PostgreSQL database connection management module
//! 
//! This module provides functionality for managing PostgreSQL database connections,
//! including connection pool initialization, configuration, and retrieval.
//! It supports connection pooling with automatic configuration based on system resources,
//! SSL configuration, and connection warmup for optimal performance.
//! 
//! PostgreSQL 数据库连接管理模块
//! 
//! 该模块提供了管理 PostgreSQL 数据库连接的功能，
//! 包括连接池初始化、配置和检索。
//! 它支持基于系统资源的自动配置连接池，
//! SSL 配置，以及连接预热以实现最佳性能。

use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions, PgSslMode};
use sqlx::Error;
use std::cmp::{max, min};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::OnceCell;
use std::time::Duration;

use crate::common::error::QueryError;

// Static database pool instance
static DB_POOL: OnceCell<Arc<PgPool>> = OnceCell::const_new();

/// Calculate connection limits based on CPU cores
/// 
/// # Returns
/// A tuple containing (max_connections, min_connections, warmup_connections)
/// 
/// 根据 CPU 核心数计算连接限制
/// 
/// # 返回值
/// 包含 (max_connections, min_connections, warmup_connections) 的元组
fn connect_limits() -> (u32, u32, u32) {
    let num_cpus = num_cpus::get() as u32;
    let max_connections = max(10, min(50, num_cpus * 3));
    let min_connections = max(2, min(10, num_cpus));    
    let warmup_connections = (max_connections as f32 * 0.2).ceil() as u32;

    (max_connections, min_connections, warmup_connections)
}

/// Initialize PostgreSQL database connection pool with custom configuration
/// 
/// # Arguments
/// * `pool` - A pre-configured PostgreSQL connection pool
/// 
/// # Returns
/// A reference to the static PostgreSQL pool or an error
/// 
/// 使用自定义配置初始化 PostgreSQL 数据库连接池
/// 
/// # 参数
/// * `pool` - 预配置的 PostgreSQL 连接池
/// 
/// # 返回值
/// 指向静态 PostgreSQL 连接池的引用或错误
pub async fn setup_db_pool(pool: PgPool) -> Result<&'static PgPool, Error> {
    // Wrap the connection pool in an Arc and initialize OnceCell
    let pool = Arc::new(pool);
    DB_POOL.get_or_try_init(|| async { Ok(pool) })
        .await
        .map(|arc| arc.as_ref())
}

/// Initialize PostgreSQL database connection pool using a database URL
/// 
/// # Arguments
/// * `database_url` - Database connection URL
/// 
/// # Returns
/// A reference to the static PostgreSQL pool or an error
/// 
/// 使用数据库 URL 初始化 PostgreSQL 数据库连接池
/// 
/// # 参数
/// * `database_url` - 数据库连接 URL
/// 
/// # 返回值
/// 指向静态 PostgreSQL 连接池的引用或错误
pub async fn create_db_pool(database_url: &str) -> Result<&'static PgPool, Error> {
    let (maxc, minc, warmupc) = connect_limits();

    let mut options = PgConnectOptions::from_str(database_url)
        .map_err(|e| Error::from(e))?;
    let ssl_mode = if database_url.contains("sslmode=disable") {
        PgSslMode::Disable
    } else if database_url.contains("sslmode=require") {
        PgSslMode::Require
    } else {
        PgSslMode::Prefer
    };
    options = options.ssl_mode(ssl_mode);

    let pool = PgPoolOptions::new()
        .max_connections(maxc)
        .min_connections(minc)
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(1800))
        //.test_before_acquire(true)
        .connect_with(options)
        .await
        .map_err(|e| Error::from(e))?;

    let _ = warmup_connect(&pool, warmupc).await;

    setup_db_pool(pool).await
}

/// Warm up database connections by acquiring and releasing them
/// 
/// # Arguments
/// * `pool` - The database connection pool
/// * `warmup_num` - Number of connections to warm up
/// 
/// # Returns
/// Ok(()) on success or an error
/// 
/// 通过获取和释放连接来预热数据库连接
/// 
/// # 参数
/// * `pool` - 数据库连接池
/// * `warmup_num` - 要预热的连接数
/// 
/// # 返回值
/// 成功时返回 Ok(()) 或错误
async fn warmup_connect(pool: &PgPool, warmup_num: u32) -> Result<(), Error> {
    for _ in 0..warmup_num {
        let conn = pool.acquire().await?;
        drop(conn);
    }
    Ok(())
}

/// Get a reference to the PostgreSQL database connection pool
/// 
/// # Returns
/// A cloned Arc reference to the PostgreSQL pool or an error if not initialized
/// 
/// 获取 PostgreSQL 数据库连接池的引用
/// 
/// # 返回值
/// PostgreSQL 连接池的克隆 Arc 引用，如果未初始化则返回错误
pub fn get_db_pool() -> Result<Arc<PgPool>, Error> {
    DB_POOL.get()
        .cloned()
        .ok_or_else(|| QueryError::DBPoolNotInitialized.into())
}