use std::future::Future;
use sqlx::{Database, Encode, Error, FromRow, Pool, Row, Type};

use super::builder::BuilderTrait;

pub trait DatabaseTrait {
    type Database: Database;
    type Row: Row;
    type QueryResult;
    type SelectQuery<'a>;

    /// Fetches a single record based on the given condition and returns the result.
    fn fetch_one<'a, T>(&self, qb: Self::SelectQuery<'a>) -> impl Future<Output = Result<T, Error>> + Send
    where
        T: for<'r> FromRow<'r, Self::Row> + Unpin + Send + 'a;

    /// Fetches multiple records based on the given condition and returns a list of results.
    fn fetch_all<'a, T>(&self, qb: Self::SelectQuery<'a>) -> impl Future<Output = Result<Vec<T>, Error>> + Send
    where
        T: for<'r> FromRow<'r, Self::Row> + Unpin + Send + 'a;

    /// Fetches an optional single record using `fetch_optional` and returns the result.
    fn fetch_optional<'a, T>(&self, qb: Self::SelectQuery<'a>) -> impl Future<Output = Result<Option<T>, Error>> + Send
    where
        T: for<'r> FromRow<'r, Self::Row> + Unpin + Send + 'a;

    /// Executes insert, update, or delete operations and automatically manages transactions.
    fn execute<'a, B, T>(&self, qb: B) -> impl Future<Output = Result<Self::QueryResult, Error>> + Send
    where
        B: BuilderTrait<T> + Send,
        T: Send + Unpin + Encode<'a, Self::Database> + Type<Self::Database> + 'a;

    /// Returns a reference to the database connection pool.
    fn get_db_pool(&self) -> &'static Pool<Self::Database>;
}