//! SQLite database connection management module
//! 
//! This module provides functionality for managing SQLite database connections,
//! including connection pool initialization, configuration, and retrieval.
//! It supports connection pooling with automatic configuration and enables
//! WAL (Write-Ahead Logging) mode for better concurrency and performance.
//! 
//! SQLite 数据库连接管理模块
//! 
//! 该模块提供了管理 SQLite 数据库连接的功能，
//! 包括连接池初始化、配置和检索。
//! 它支持连接池的自动配置，并启用 WAL（预写日志）模式
//! 以获得更好的并发性和性能。

use sqlx::{Pool, Sqlite};
use sqlx::{pool::PoolOptions, Error, SqlitePool};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::OnceCell;
use std::time::Duration;

use crate::common::error::QueryError;

// Global static variable to store the database connection pool
static DB_POOL: OnceCell<Arc<SqlitePool>> = OnceCell::const_new();

/// Initialize the connection pool with a custom pool
/// 
/// # Arguments
/// * `pool` - A pre-configured SQLite connection pool
/// 
/// # Returns
/// A reference to the static SQLite pool or an error
/// 
/// 使用自定义连接池初始化连接池
/// 
/// # 参数
/// * `pool` - 预配置的 SQLite 连接池
/// 
/// # 返回值
/// 指向静态 SQLite 连接池的引用或错误
pub async fn setup_db_pool<'a>(pool: Pool<Sqlite>) -> Result<&'a SqlitePool, Error> {
    // Create the connection pool
    let pool = Arc::new(pool);

    // Force evaluation of OnceCell to ensure the connection pool is initialized
    DB_POOL.get_or_try_init(|| async { Ok(pool) }).await
        .map(|arc| arc.as_ref())
}

/// Initializes the database connection pool with the database URL and enables WAL mode
/// 
/// # Arguments
/// * `database_url` - Database connection URL
/// 
/// # Returns
/// A reference to the static SQLite pool or an error
/// 
/// 使用数据库 URL 初始化数据库连接池并启用 WAL 模式
/// 
/// # 参数
/// * `database_url` - 数据库连接 URL
/// 
/// # 返回值
/// 指向静态 SQLite 连接池的引用或错误
pub async fn create_db_pool(database_url: &str) -> Result<&SqlitePool, Error> {

    let connect_options = SqliteConnectOptions::from_str(database_url)
        .map_err(|e| Error::from(e))?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(Duration::from_secs(8));

    let pool = PoolOptions::new()
        .max_connections(8)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(30))
        .test_before_acquire(false)
        .connect_with(connect_options)
        .await
        .map_err(|e| Error::from(e))?;

    setup_db_pool(pool).await
}

/// Gets a reference to the database connection pool
/// 
/// # Returns
/// A cloned Arc reference to the SQLite pool or an error if not initialized
/// 
/// 获取数据库连接池的引用
/// 
/// # 返回值
/// SQLite 连接池的克隆 Arc 引用，如果未初始化则返回错误
pub fn get_db_pool() -> Result<Arc<SqlitePool>, Error> {
    DB_POOL.get()
        .cloned() // Clone the Arc to return a new reference
        .ok_or_else(||QueryError::DBPoolNotInitialized.into())
}