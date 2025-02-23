use std::future::Future;
use serde::{Deserialize, Serialize};
use sqlx::{Database, Error, FromRow};

pub trait OperationsTrait<'a, T, DB>: Send + Sync 
where
    DB: Database,
    T: for<'r> FromRow<'r, DB::Row> + Send + Sync + Default,
{
    type Query;
    type DataKind;
    type QueryResult;

    /// 创建一个新的 `Operations` 实例。
    /// # 参数
    /// * `table_name`: 表名。
    /// * `primary_key`: 主键名。
    /// * `soft_delete_info`: 软删除信息，包括软删除字段名（有即视为软删除）、查询语句是否启用软删除过滤。
    /// 
    /// # 返回值
    /// 返回一个新的 `Operations` 实例。
    ///
    fn new(table_name: &'a str, primary_key: &'a str, soft_delete_info: Option<(&'a str, bool)>) -> Self;

    /// 插入一条记录到数据库中，并返回插入记录的主键值。
    fn insert_one(&self, entity: T) -> impl Future<Output = Result<Self::QueryResult, Error>> + Send;

    /// 插入多条记录到数据库中，并返回插入记录的主键值列表。
    fn insert_many(&self, entities: Vec<T>) -> impl Future<Output = Result<Self::QueryResult, Error>> + Send;

    /// 更新一条记录，并返回受影响的行数。
    fn update_one(&self, entity: T, override_empty: bool) -> impl Future<Output = Result<Self::QueryResult, Error>> + Send;

    /// 更新多条记录，并返回受影响的行数。
    fn update_many(&self, entities: Vec<T>, override_empty: bool) -> impl Future<Output = Result<Vec<Self::QueryResult>, Error>> + Send;

    /// 删除一条记录，并返回受影响的行数。
    fn delete_one(&self, key: impl Into<Self::DataKind> + Send) -> impl Future<Output = Result<Self::QueryResult, Error>> + Send;

    /// 删除多条记录，并返回受影响的行数。
    fn delete_many(&self, keys: Vec<impl Into<Self::DataKind> + Send>) -> impl Future<Output = Result<Self::QueryResult, Error>> + Send;

    /// 查询并返回表中的所有记录，支持条件查询。
    fn fetch_all(&self, query_condition: Self::Query) -> impl Future<Output = Result<Vec<T>, Error>> + Send;

    /// 根据主键查询并返回一条记录。
    fn fetch_by_key(&self, id: impl Into<Self::DataKind> + Send) -> impl Future<Output = Result<Option<T>, Error>> + Send;

    /// 根据字段条件查询并返回一条记录。
    fn fetch_one(&self, query_condition: Self::Query) -> impl Future<Output = Result<Option<T>, Error>> + Send;

    /// 分页查询并返回表中的记录，支持条件查询。
    fn fetch_paginated(&self, page_number: u64, page_size: u64, query_condition: Self::Query) -> impl Future<Output = Result<PaginatedResult<T>, Error>> + Send;

    /// 游标分页查询，并返回表中的记录，支持条件查询。
    fn fetch_by_cursor(&self, limit: u64, query_condition: Self::Query) -> impl Future<Output = Result<CursorPaginatedResult<T>, Error>> + Send  where T: Clone;

    /// 检查某个字段的值是否唯一。
    fn exist(&self, query_condition: Self::Query) -> impl Future<Output = Result<bool, Error>> + Send;

    /// 获取记录总数，支持条件查询。
    fn count(&self, query_condition: Self::Query) -> impl Future<Output = Result<i64, Error>> + Send;

    /// 恢复一条软删除的记录。
    fn restore_one(&self, key: impl Into<Self::DataKind> + Send) -> impl Future<Output = Result<Self::QueryResult, Error>> + Send;

    /// 恢复多条软删除的记录。
    fn restore_many(&self, keys: Vec<impl Into<Self::DataKind> + Send>) -> impl Future<Output = Result<Self::QueryResult, Error>> + Send;

}

/// 分页查询结果结构体。
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct PaginatedResult<T> {
    /// 查询到的数据记录。
    pub data: Vec<T>,
    /// 总记录数。
    pub total: i64,
    pub page_number: u64,
    pub page_size: u64,
}

/// 游标分页结果结构体
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CursorPaginatedResult<T> {
    pub data: Vec<T>,      // 分页数据
    pub next_cursor: Option<T>, // 下一个游标值
    pub page_size: u64,
}
