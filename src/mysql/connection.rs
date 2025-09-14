//! MySQL database connection management module
//! 
//! This module provides functionality for managing MySQL database connections,
//! including connection pool initialization, configuration, and retrieval.
//! It supports connection pooling with automatic configuration based on system resources,
//! SSL configuration, and connection warmup for optimal performance.
//! 
//! # 中文
//! MySQL 数据库连接管理模块
//! 
//! 该模块提供了管理 MySQL 数据库连接的功能，
//! 包括连接池初始化、配置和检索。
//! 它支持基于系统资源的自动配置连接池，
//! SSL 配置，以及连接预热以实现最佳性能。

use crate::common::error::QueryError;

use sqlx::{Pool, MySql};
use sqlx::{pool::PoolOptions, Error, MySqlPool};
use sqlx::mysql::{MySqlConnectOptions, MySqlSslMode};
use std::cmp::{max, min};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::OnceCell;
use std::time::Duration;

static DB_POOL: OnceCell<Arc<MySqlPool>> = OnceCell::const_new();

/// Calculate connection limits based on CPU cores
/// 
/// # Returns
/// A tuple containing (max_connections, min_connections, warmup_connections)
/// 
/// # 中文
/// 根据 CPU 核心数计算连接限制
/// 
/// # 返回值
/// 包含 (max_connections, min_connections, warmup_connections) 的元组
fn connect_limits() -> (u32, u32, u32) {
    let num_cpus = num_cpus::get() as u32;
    let max_connections = max(10, min(50, num_cpus * 2));
    let min_connections = max(2, min(10, num_cpus / 2));
    let warmup = (max_connections as f32 * 0.2).ceil() as u32;

    (max_connections, min_connections, warmup)
}

/// Initializes the database connection pool with custom settings
/// 
/// # Arguments
/// * `pool` - A pre-configured MySQL connection pool
/// 
/// # Returns
/// A reference to the static MySQL pool or an error
/// 
/// # 中文
/// 使用自定义设置初始化数据库连接池
/// 
/// # 参数
/// * `pool` - 预配置的 MySQL 连接池
/// 
/// # 返回值
/// 指向静态 MySQL 连接池的引用或错误
pub async fn setup_db_pool(pool: Pool<MySql>) -> Result<&'static MySqlPool, Error> {
    // Create the connection pool
    let pool = Arc::new(pool);

    // Force initialization of OnceCell to ensure the connection pool is initialized
    DB_POOL.get_or_try_init(|| async { Ok(pool) }).await
        .map(|arc| arc.as_ref())
}

/// Initializes the database connection pool with a database URL
/// 
/// # Arguments
/// * `database_url` - Database connection URL
/// 
/// # Returns
/// A reference to the static MySQL pool or an error
/// 
/// # 中文
/// 使用数据库 URL 初始化数据库连接池
/// 
/// # 参数
/// * `database_url` - 数据库连接 URL
/// 
/// # 返回值
/// 指向静态 MySQL 连接池的引用或错误
pub async fn create_db_pool(database_url: &str) -> Result<&'static MySqlPool, Error> {
    let (maxc, minc, warmupc) = connect_limits();
    let mut options = MySqlConnectOptions::from_str(database_url)
        .map_err(|e| Error::from(e))?;

    let ssl_mode = if database_url.contains("sslmode=disable") {
        MySqlSslMode::Disabled
    } else if database_url.contains("sslmode=require") {
        MySqlSslMode::Required
    } else {
        MySqlSslMode::Preferred
    };
    options = options.ssl_mode(ssl_mode);

    let pool = PoolOptions::new()
        .max_connections(maxc)
        .min_connections(minc)
        .acquire_timeout(Duration::from_secs(5))
        .test_before_acquire(false)
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(1800))
        //.test_before_acquire(false)
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
/// # 中文
/// 通过获取和释放连接来预热数据库连接
/// 
/// # 参数
/// * `pool` - 数据库连接池
/// * `warmup_num` - 要预热的连接数
/// 
/// # 返回值
/// 成功时返回 Ok(()) 或错误
async fn warmup_connect(pool: &MySqlPool, warmup_num: u32) -> Result<(), Error> {
    for _ in 0..warmup_num {
        let conn = pool.acquire().await?;
        drop(conn);
    }
    Ok(())
}

/// Gets a reference to the database connection pool
/// 
/// # Returns
/// A cloned Arc reference to the MySQL pool or an error if not initialized
/// 
/// # 中文
/// 获取数据库连接池的引用
/// 
/// # 返回值
/// MySQL 连接池的克隆 Arc 引用，如果未初始化则返回错误
pub fn get_db_pool() -> Result<Arc<MySqlPool>, Error> {
    DB_POOL.get()
        .cloned()
        .ok_or_else(||QueryError::DBPoolNotInitialized.into())
}