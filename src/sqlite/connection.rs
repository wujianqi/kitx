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

    // 强制评估 OnceCell，确保连接池被初始化
    DB_POOL.get_or_try_init(|| async { Ok(pool.clone()) }).await
        .map(|arc| arc.as_ref())
}

/// 获取数据库连接池的引用。
pub fn get_db_pool() -> &'static Arc<SqlitePool> {
    DB_POOL.get().expect("Database pool not initialized")
}
