use std::{future::Future, sync::Arc};
use sqlx::{Database, Error, FromRow, Pool};

use super::builder::BuilderTrait;

pub trait QueryExecutor<D, DB> 
where
    DB: Database,
{
    /// Fetches a single record based on the given condition and returns the result.
    fn fetch_one<T, B>(&self, qb: B) -> impl Future<Output = Result<T, Error>> + Send
    where
        T: for<'r> FromRow<'r, DB::Row> + Unpin + Send,
        B: BuilderTrait<D> + Send + Sync;

    /// Fetches multiple records based on the given condition and returns a list of results.
    fn fetch_all<T, B>(&self, qb: B) -> impl Future<Output = Result<Vec<T>, Error>> + Send
    where
        T: for<'r> FromRow<'r, DB::Row> + Unpin + Send,
        B: BuilderTrait<D> + Send + Sync;

    /// Fetches an optional single record using `fetch_optional` and returns the result.
    fn fetch_optional<T, B>(&self, qb: B) -> impl Future<Output = Result<Option<T>, Error>> + Send
    where
        T: for<'r> FromRow<'r, DB::Row> + Unpin + Send,
        B: BuilderTrait<D> + Send + Sync;

    /// Executes insert, update, or delete operations and automatically manages transactions.
    fn execute<B>(&self, qb: B) -> impl Future<Output = Result<DB::QueryResult, Error>> + Send
    where       
        B: BuilderTrait<D> + Send + Sync;

    /// Returns a reference to the database connection pool.
    fn get_db_pool(&self) -> Result<Arc<Pool<DB>>, Error>;
}