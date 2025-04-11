use std::sync::Arc;

use sqlx::mysql::{MySqlQueryResult, MySqlRow};
use sqlx::{Acquire, Error, FromRow, Pool, MySql};

use crate::common::builder::BuilderTrait;
use crate::common::query::QueryExecutor;
use crate::common::error::OperationError;
use super::connection;
use super::kind::DataKind;

pub struct MySqlQuery;

impl<'a> QueryExecutor<DataKind<'a>, MySql> for MySqlQuery {
    async fn fetch_one<T, B>(&self, qb: B) -> Result<T, Error>
    where
        T: for<'r> FromRow<'r, MySqlRow> + Unpin + Send,
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
        T: for<'r> FromRow<'r, MySqlRow> + Unpin + Send,
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
        T: for<'r> FromRow<'r, MySqlRow> + Unpin + Send,
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

    async fn execute<B>(&self, qb: B) -> Result<MySqlQueryResult, Error>
    where
        B: BuilderTrait<DataKind<'a>> + Send + Sync,
    {
        let (sql, values) = qb.build();
        if values.is_empty() {
            return Err(OperationError::db("No parameters provided".to_string()).into());
        }
        let pool = self.get_db_pool()?;
        let mut conn = pool.acquire().await?;
        let mut tx = conn.begin().await?;

        let sql_str: &str = Box::leak(sql.into_boxed_str());
        let mut query = sqlx::query(sql_str);

        for value in values {
            query = query.bind(value);
        }

        match query.execute(&mut *tx).await {
            Ok(r) => {
                tx.commit().await?;
                Ok(r)
            }
            Err(e) => {
                tx.rollback().await?;
                Err(e)
            }
        }
    }

    fn get_db_pool(&self) -> Result<Arc<Pool<MySql>>, Error> {
        connection::get_db_pool()
    }
}