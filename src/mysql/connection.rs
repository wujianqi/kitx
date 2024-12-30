use sqlx::{Pool, MySql};
use sqlx::{pool::PoolOptions, Error, MySqlPool};
use sqlx::mysql::MySqlConnectOptions;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::OnceCell;
use std::time::Duration;

// 全局静态变量，用于存储数据库连接池
static DB_POOL: OnceCell<Arc<MySqlPool>> = OnceCell::const_new();

/// 按自定义的 Pool 初始化连接池
pub async fn init_db_pool_custom(pool: Pool<MySql>) -> Result<&'static MySqlPool, Error> {
    // 创建连接池
    let pool = Arc::new(pool);

    // 强制评估 OnceCell，确保连接池被初始化
    DB_POOL.get_or_try_init(|| async { Ok(pool.clone()) }).await
        .map(|arc| arc.as_ref())
}

/// 按数据库地址初始化数据库连接池
pub async fn init_db_pool(database_url: &str) -> Result<&'static MySqlPool, Error> {
    // 配置 MySQL 连接选项
    let connect_options = MySqlConnectOptions::from_str(database_url)?
        .ssl_mode(sqlx::mysql::MySqlSslMode::Disabled); // 根据需要配置 SSL 模式

    // 创建连接池
    let pool = PoolOptions::new()
        .max_connections(50) // 根据负载调整最大连接数
        .min_connections(10) // 预先建立最小连接数，减少首次请求的延迟
        .acquire_timeout(Duration::from_secs(5)) // 获取连接的超时时间
        .idle_timeout(Duration::from_secs(300)) // 空闲连接的超时时间
        .max_lifetime(Duration::from_secs(1800)) // 连接的最大生命周期
        .test_before_acquire(true) // 获取连接前测试连接是否有效
        .connect_with(connect_options)
        .await?;

    // 预热连接池，预先建立最小连接数
    let _ = pool.acquire().await;

    // 初始化连接池
    init_db_pool_custom(pool).await
}

/// 获取数据库连接池的引用
pub fn get_db_pool() -> &'static MySqlPool {
    DB_POOL.get().expect("Database pool not initialized")
}
