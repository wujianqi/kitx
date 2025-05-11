use std::mem::take;
use std::sync::Arc;

use sqlx::sqlite::{SqliteQueryResult, SqliteRow};
use sqlx::{Acquire, Error, FromRow, Pool, Sqlite};
use tokio::sync::Mutex;

use crate::common::builder::BuilderTrait;
use crate::common::query::QueryExecutor;
use crate::utils::query_condition::Shared;
use super::connection;
use super::kind::DataKind;

pub struct SqliteQuery<'a>  {
    is_transaction_active: Mutex<bool>,
    pending_statements: Mutex<Vec<(String, Vec<DataKind<'a>>)>>
}

impl<'a>  SqliteQuery<'a>  {
    pub fn new() -> Self {
        SqliteQuery {
            is_transaction_active: Mutex::new(false),
            pending_statements: Mutex::new(vec![])
        }
    }

    pub fn shared() -> Shared<SqliteQuery<'a>> {
        Shared::new(Self::new())
    }

    async fn execute_with_trans(&self, 
        pending_statements: Vec<(String, Vec<DataKind<'a>>)>) -> Result<Vec<SqliteQueryResult>, Error>
    {
        let pool = self.get_db_pool()?;
        let mut conn = pool.acquire().await?;
        let mut tx = conn.begin().await?;
        let mut results = Vec::new();

        for ps in pending_statements {
            let (sql, values) = ps;
            let mut query = sqlx::query(&sql);
            for value in values {
                query = query.bind(value);
            }
            match query.execute(&mut *tx).await {
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

    pub async fn begin_transaction(&self) -> Result<&Self, Error> {
        *self.is_transaction_active.lock().await = true;
        Ok(self)
    }

    pub async fn commit(&self) -> Result<Vec<SqliteQueryResult>, Error> {
        let builders = {
            let mut stmts = self.pending_statements.lock().await;
            take(&mut *stmts)
        };
        *self.is_transaction_active.lock().await = false;
        self.execute_with_trans(builders).await
    }
}

impl<'a> QueryExecutor<DataKind<'a>, Sqlite> for SqliteQuery<'a> {
    async fn fetch_one<T, B>(&self, qb: B) -> Result<T, Error>
    where
        T: for<'r> FromRow<'r, SqliteRow> + Unpin + Send,
        B: BuilderTrait<DataKind<'a>> + Send + Sync,
    {
        let (sql, values) = qb.build();
        let pool = self.get_db_pool()?;
        let mut query = sqlx::query_as::<_, T>(&sql);

        // Bind parameter values to the query
        for value in values {
            query = query.bind(value);
        }

        // Execute the query and return a single record
        query.fetch_one(&*pool).await
    }

    async fn fetch_all<T, B>(&self, qb: B) -> Result<Vec<T>, Error>
    where
        T: for<'r> FromRow<'r, SqliteRow> + Unpin + Send,
        B: BuilderTrait<DataKind<'a>> + Send + Sync,
    {
        let pool = self.get_db_pool()?;
        let (sql, values) = qb.build();
        let mut query = sqlx::query_as::<_, T>(&sql);

        // Bind parameter values to the query
        for value in values {
            query = query.bind(value);
        }

        // Execute the query and return multiple records
        query.fetch_all(&*pool).await
    }

    async fn fetch_optional<T, B>(&self, qb: B) -> Result<Option<T>, Error>
    where
        T: for<'r> FromRow<'r, SqliteRow> + Unpin + Send,
        B: BuilderTrait<DataKind<'a>> + Send + Sync,
    {
        let pool = self.get_db_pool()?;
        let (sql, values) = qb.build();

        let mut query = sqlx::query_as::<_, T>(&sql);

        // Bind parameter values to the query
        for value in values {
            query = query.bind(value);
        }

        // Execute the query and return a single optional record
        query.fetch_optional(&*pool).await
    }

    async fn execute<B>(&self, qb: B) -> Result<SqliteQueryResult, Error>
    where
        B: BuilderTrait<DataKind<'a>> + Send + Sync,
    {
        if *self.is_transaction_active.lock().await {
            self.pending_statements.lock().await.push(qb.build());
            Ok(SqliteQueryResult::default())
        } else {
            let pool = self.get_db_pool()?;
            let (sql, values) = qb.build();
            let mut query = sqlx::query(&sql);
            for value in values {
                query = query.bind(value);
            }
            query.execute(&*pool).await
        }
    }

    fn get_db_pool(&self) -> Result<Arc<Pool<Sqlite>>, Error> {
        connection::get_db_pool()
    }

}
