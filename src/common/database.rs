use std::future::Future;
use sqlx::{Database, Error, FromRow, Pool, Row};

pub trait DatabaseTrait {
    type Database: Database;
    type Row: Row;
    type QueryResult;
    //&'a (dyn Encode<'a, Self::Database> + Send + Sync);
    type QueryBuilder<'a>; 

    /// 根据条件查询单条记录，并返回结果。
    fn fetch_one<'a, T>(&self, qb: Self::QueryBuilder<'a>) -> impl Future<Output = Result<T, Error>> + Send
    where
        T: for<'r> FromRow<'r, Self::Row> + Unpin + Send;

    /// 根据条件查询多条记录，并返回结果列表。
    fn fetch_all<'a, T>(&self, qb: Self::QueryBuilder<'a>) -> impl Future<Output = Result<Vec<T>, Error>> + Send
    where
        T: for<'r> FromRow<'r, Self::Row> + Unpin + Send;

    /// 使用 `fetch_optional` 查询单条可选记录，并返回结果。
    fn fetch_optional<'a, T>(&self, qb: Self::QueryBuilder<'a>) -> impl Future<Output = Result<Option<T>, Error>> + Send
    where
        T: for<'r> FromRow<'r, Self::Row> + Unpin + Send;

    /// 增删改数据，并自动管理事务。
    fn execute<'a>(&self, qb: Self::QueryBuilder<'a>) -> impl Future<Output = Result<Self::QueryResult, Error>> + Send;

    /// 获取数据库连接池。
    fn get_db_pool(&self) -> &'static Pool<Self::Database>;

}
