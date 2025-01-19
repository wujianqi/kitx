use sqlx::sqlite::{SqliteQueryResult, SqliteRow};
use sqlx::{Acquire, Error, FromRow};

use crate::sqlite_field_bind;
use super::connection;
use super::sql::SQLBuilder;

/// 根据条件查询单条记录，并返回结果。
///
/// # 参数
/// - `sb`: SQL 构建器，包含构建的 SQL 语句和参数值。
///
/// # 返回
/// - `Result<T, Error>`: 如果查询成功，则返回一个 `T` 类型的结果；如果查询失败，则返回一个错误。
pub async fn fetch_one<'a, T>(sb: SQLBuilder<'a>) -> Result<T, Error>
where
    T: for<'r> FromRow<'r, SqliteRow> + Unpin + Send,
{
    let pool = connection::get_db_pool();
    let (sql, values) = sb.build();
    let mut query = sqlx::query_as::<_, T>(&sql);

    // 绑定参数值到查询中
    for value in values {
        query = sqlite_field_bind!(query, value);
    }

    // 执行查询并返回单条记录
    query.fetch_one(&**pool).await
}

/// 根据条件查询多条记录，并返回结果列表。
///
/// # 参数
/// - `sb`: SQL 构建器，包含构建的 SQL 语句和参数值。
///
/// # 返回
/// - `Result<Vec<T>, Error>`: 如果查询成功，则返回一个 `Vec<T>` 类型的结果列表；如果查询失败，则返回一个错误。
pub async fn fetch_all<'a, T>(sb: SQLBuilder<'a>) -> Result<Vec<T>, Error>
where
    T: for<'r> FromRow<'r, SqliteRow> + Unpin + Send,
{
    let pool = connection::get_db_pool();
    let (sql, values) = sb.build();
    let mut query = sqlx::query_as::<_, T>(&sql);

    // 绑定参数值到查询中
    for value in values {
        query = sqlite_field_bind!(query, value);
    }

    // 执行查询并返回多条记录
    query.fetch_all(&**pool).await
}

/// 使用 `fetch_optional` 查询单条可选记录，并返回结果。
///
/// # 参数
/// - `sb`: SQL 构建器，包含构建的 SQL 语句和参数值。
///
/// # 返回
/// - `Result<Option<T>, Error>`: 如果查询成功，则返回一个 `Option<T>` 类型的结果（可能为 `Some(T)` 或 `None`）；如果查询失败，则返回一个错误。
pub async fn fetch_optional<'a, T>(sb: SQLBuilder<'a>) -> Result<Option<T>, Error>
where
    T: for<'r> FromRow<'r, SqliteRow> + Unpin + Send,
{
    let pool = connection::get_db_pool();
    let (sql, values) = sb.build();
    let mut query = sqlx::query_as::<_, T>(&sql);

    // 绑定参数值到查询中
    for value in values {
        query = sqlite_field_bind!(query, value);
    }

    // 执行查询并返回单条可选记录
    query.fetch_optional(&**pool).await
}

/// 增删改数据，并自动管理事务。
///
/// # 参数
/// - `sb`: SQL 构建器，包含构建的 SQL 语句和参数值。
///
/// # 返回
/// - `Result<SqliteQueryResult, Error>`: 如果执行成功，则返回一个 `SqliteQueryResult` 结果；如果执行失败，则返回一个错误。
pub async fn execute<'a>(sb: SQLBuilder<'a>) -> Result<SqliteQueryResult, Error> {
    let pool = connection::get_db_pool();
    let mut conn = pool.acquire().await?;
    let mut tx = conn.begin().await?;
    let (sql, values) = sb.build();
    let mut query = sqlx::query(&sql);

    // 绑定参数值到查询中
    for value in values {
        query = sqlite_field_bind!(query, value);
    }

    // 执行查询并处理事务
    let result = query.execute(&mut *tx).await;

    match result {
        Ok(r) => {
            // 提交事务
            tx.commit().await?;
            Ok(r)
        },
        Err(e) => {
            // 回滚事务
            tx.rollback().await?;
            Err(e)
        }
    }
}
