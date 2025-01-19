<<<<<<< HEAD
use sqlx::{Pool, Sqlite};
use sqlx::{pool::PoolOptions, Error, SqlitePool};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::OnceCell;
use std::time::Duration;

// 全局静态变量，用于存储数据库连接池
static DB_POOL: OnceCell<Arc<SqlitePool>> = OnceCell::const_new();

// 按自定义的pool初始化连接池
pub async fn init_db_pool_custom<'a>(pool: Pool<Sqlite>) -> Result<&'a SqlitePool, Error> {
    // 创建连接池
    let pool = Arc::new(pool);
=======
use sqlx::{pool::PoolOptions, Error, SqlitePool};
use tokio::sync::OnceCell;
use std::{sync::Arc, time::Duration};

static DB_POOL: OnceCell<Arc<SqlitePool>> = OnceCell::const_new();

#[allow(dead_code)]
/// 初始化数据库连接池，并将其存储为全局静态变量。
pub async fn init_db_pool(database_url: &str) -> Result<&SqlitePool, Error> {
    let pool = Arc::new(
        PoolOptions::new()
            .max_connections(20)
            .min_connections(5)
            .acquire_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(30))
            .connect(database_url)
            .await?,
    );
>>>>>>> b0e16200fd5be220e3fbdfdf161f92205821e722

    // 强制评估 OnceCell，确保连接池被初始化
    DB_POOL.get_or_try_init(|| async { Ok(pool.clone()) }).await
        .map(|arc| arc.as_ref())
}

<<<<<<< HEAD
/// 按数据库地址初始化数据库连接池，并启用 WAL 模式。
pub async fn init_db_pool(database_url: &str) -> Result<&SqlitePool, Error> {
    // 配置 SQLite 连接选项
    let connect_options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true) // 如果数据库不存在则创建
        .journal_mode(SqliteJournalMode::Wal); // 启用 WAL 模式

    // 创建连接池
    let pool = PoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(30))
        .connect_with(connect_options)
        .await?;

    init_db_pool_custom(pool).await
}

=======
>>>>>>> b0e16200fd5be220e3fbdfdf161f92205821e722
/// 获取数据库连接池的引用。
pub fn get_db_pool() -> &'static Arc<SqlitePool> {
    DB_POOL.get().expect("Database pool not initialized")
}
