use std::future::Future;
use sqlx::{Database, Error, FromRow, Pool, Row};

pub trait DatabaseTrait {
    type Database: Database;
    type Row: Row;
    type QueryResult;
    //&'a (dyn Encode<'a, Self::Database> + Send + Sync);
    type QueryBuilder<'a>; 

    /// Fetches a single record based on the given condition and returns the result.
    fn fetch_one<'a, T>(&self, qb: Self::QueryBuilder<'a>) -> impl Future<Output = Result<T, Error>> + Send
    where
        T: for<'r> FromRow<'r, Self::Row> + Unpin + Send;

    /// Fetches multiple records based on the given condition and returns a list of results.
    fn fetch_all<'a, T>(&self, qb: Self::QueryBuilder<'a>) -> impl Future<Output = Result<Vec<T>, Error>> + Send
    where
        T: for<'r> FromRow<'r, Self::Row> + Unpin + Send;

    /// Fetches an optional single record using `fetch_optional` and returns the result.
    fn fetch_optional<'a, T>(&self, qb: Self::QueryBuilder<'a>) -> impl Future<Output = Result<Option<T>, Error>> + Send
    where
        T: for<'r> FromRow<'r, Self::Row> + Unpin + Send;

    /// Executes insert, update, or delete operations and automatically manages transactions.
    fn execute<'a>(&self, qb: Self::QueryBuilder<'a>) -> impl Future<Output = Result<Self::QueryResult, Error>> + Send;

    /// Returns a reference to the database connection pool.
    fn get_db_pool(&self) -> &'static Pool<Self::Database>;
}