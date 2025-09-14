//! MySQL database query execution module
//! 
//! This module provides functions for executing various types of database queries
//! against a MySQL database. It includes functions for executing queries, fetching
//! single or multiple rows, and handling transactions. All functions are designed
//! to work with the MySQL-specific sqlx types.
//! 
//! # 中文
//! MySQL 数据库查询执行模块
//! 
//! 该模块提供了针对 MySQL 数据库执行各种类型数据库查询的函数。
//! 它包括执行查询、获取单行或多行数据以及处理事务的函数。
//! 所有函数都设计为与 MySQL 特定的 sqlx 类型配合使用。

use sqlx::{mysql::{MySqlQueryResult, MySqlRow}, Acquire, Error, FromRow, QueryBuilder, MySql};

use crate::mysql::connection;

/// Execute a query and return the result
/// 
/// # Arguments
/// * `builder` - QueryBuilder containing the query to execute
/// 
/// # Returns
/// MySqlQueryResult on success or an Error
/// 
/// # 中文
/// 执行查询并返回结果
/// 
/// # 参数
/// * `builder` - 包含要执行查询的 QueryBuilder
/// 
/// # 返回值
/// 成功时返回 MySqlQueryResult，失败时返回 Error
pub async fn execute<'a>(
    mut builder: QueryBuilder<'a, MySql>,
) -> Result<MySqlQueryResult, Error>
{
    #[cfg(debug_assertions)]
    {
        let sql = builder.sql();
        dbg!(sql);
    }
    let pool = connection::get_db_pool()?;
    builder.build().execute(&*pool).await
}

/// Execute multiple queries within a transaction
/// 
/// # Arguments
/// * `builders` - Vector of QueryBuilders containing the queries to execute
/// 
/// # Returns
/// Vector of MySqlQueryResults on success or an Error
/// 
/// # 中文
/// 在事务中执行多个查询
/// 
/// # 参数
/// * `builders` - 包含要执行查询的 QueryBuilder 向量
/// 
/// # 返回值
/// 成功时返回 MySqlQueryResult 向量，失败时返回 Error
pub async fn execute_with_trans<'a>(
    builders: Vec<QueryBuilder<'a, MySql>>,
) -> Result<Vec<MySqlQueryResult>, Error>
{
    #[cfg(debug_assertions)]
    {
        for builder in builders.iter() {
            let sql = builder.sql();
            dbg!(sql);
        }
    }
    let pool = connection::get_db_pool()?;
    let mut conn = pool.acquire().await?;
    let mut tx = conn.begin().await?;
    let mut results = Vec::new();

    for mut builder in builders {
        match builder.build().execute(&mut *tx).await {
            Ok(result) => {
                results.push(result);
            }
            Err(e) => {
                tx.rollback().await?;
                return Err(e);
            }
        }
    }

    tx.commit().await?;
    Ok(results)
}

/// Fetch an optional single row and map it to a type
/// 
/// # Type Parameters
/// * `T` - Type to map the row to, must implement FromRow trait
/// 
/// # Arguments
/// * `builder` - QueryBuilder containing the query to execute
/// 
/// # Returns
/// Optional mapped type on success or an Error
/// 
/// # 中文
/// 获取可选的单行数据并映射到类型
/// 
/// # 类型参数
/// * `T` - 要映射到的类型，必须实现 FromRow trait
/// 
/// # 参数
/// * `builder` - 包含要执行查询的 QueryBuilder
/// 
/// # 返回值
/// 成功时返回可选的映射类型，失败时返回 Error
pub async fn fetch_optional<'a, T>(
    mut builder: QueryBuilder<'a, MySql>,
) -> Result<Option<T>, Error>
where
    T: for<'r> FromRow<'r, MySqlRow> + Unpin + Send + 'a,
{
    #[cfg(debug_assertions)]
    {
        let sql = builder.sql();
        dbg!(sql);
    }
    let pool = connection::get_db_pool()?;
    builder.build_query_as::<T>().fetch_optional(&*pool).await
}

/// Fetch a single row and map it to a type
/// 
/// # Type Parameters
/// * `T` - Type to map the row to, must implement FromRow trait
/// 
/// # Arguments
/// * `builder` - QueryBuilder containing the query to execute
/// 
/// # Returns
/// Mapped type on success or an Error
/// 
/// # 中文
/// 获取单行数据并映射到类型
/// 
/// # 类型参数
/// * `T` - 要映射到的类型，必须实现 FromRow trait
/// 
/// # 参数
/// * `builder` - 包含要执行查询的 QueryBuilder
/// 
/// # 返回值
/// 成功时返回映射类型，失败时返回 Error
pub async fn fetch_one<'a, T>(
    mut builder: QueryBuilder<'a, MySql>,
) -> Result<T, Error>
where
    T: for<'r> FromRow<'r, MySqlRow> + Unpin + Send + 'a,
{
    #[cfg(debug_assertions)]
    {
        let sql = builder.sql();
        dbg!(sql);
    }
    let pool = connection::get_db_pool()?;
    builder.build_query_as::<T>().fetch_one(&*pool).await
}

/// Fetch all rows and map them to a vector of types
/// 
/// # Type Parameters
/// * `T` - Type to map the rows to, must implement FromRow trait
/// 
/// # Arguments
/// * `builder` - QueryBuilder containing the query to execute
/// 
/// # Returns
/// Vector of mapped types on success or an Error
/// 
/// # 中文
/// 获取所有行数据并映射到类型向量
/// 
/// # 类型参数
/// * `T` - 要映射到的类型，必须实现 FromRow trait
/// 
/// # 参数
/// * `builder` - 包含要执行查询的 QueryBuilder
/// 
/// # 返回值
/// 成功时返回映射类型的向量，失败时返回 Error
pub async fn fetch_all<'a, T>(
    mut builder: QueryBuilder<'a, MySql>,
) -> Result<Vec<T>, Error>
where
    T: for<'r> FromRow<'r, MySqlRow> + Unpin + Send + 'a,
{
    #[cfg(debug_assertions)]
    {
        let sql = builder.sql();
        dbg!(sql);
    }
    let pool = connection::get_db_pool()?;
    builder.build_query_as::<T>().fetch_all(&*pool).await
}

/// Fetch a scalar value (typically a count or id)
/// 
/// # Arguments
/// * `builder` - QueryBuilder containing the query to execute
/// 
/// # Returns
/// u64 scalar value on success or an Error
/// 
/// # 中文
/// 获取标量值（通常是计数或ID）
/// 
/// # 参数
/// * `builder` - 包含要执行查询的 QueryBuilder
/// 
/// # 返回值
/// 成功时返回 u64 标量值，失败时返回 Error
pub async fn fetch_scalar<'a>(
    mut builder: QueryBuilder<'a, MySql>
) -> Result<i64, Error>
{
    #[cfg(debug_assertions)]
    {
        let sql = builder.sql();
        dbg!(sql);
    }
    let pool = connection::get_db_pool()?;
    builder.build_query_scalar::<i64>().fetch_one(&*pool).await
}

/// Fetch an optional scalar value (typically a count or id)
/// 
/// # Arguments
/// * `builder` - QueryBuilder containing the query to execute
/// 
/// # Returns
/// Optional u64 scalar value on success or an Error
/// 
/// # 中文
/// 获取可选的标量值（通常是计数或ID）
/// 
/// # 参数
/// * `builder` - 包含要执行查询的 QueryBuilder
/// 
/// # 返回值
/// 成功时返回可选的 u64 标量值，失败时返回 Error
pub async fn fetch_scalar_optional<'a>(
    mut builder: QueryBuilder<'a, MySql>,
) -> Result<Option<i64>, Error>
{
    #[cfg(debug_assertions)]
    {
        let sql = builder.sql();
        dbg!(sql);
    }
    let pool = connection::get_db_pool()?;
    builder.build_query_scalar::<i64>().fetch_optional(&*pool).await
}