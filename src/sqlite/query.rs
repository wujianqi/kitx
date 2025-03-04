use sqlx::sqlite::{SqliteQueryResult, SqliteRow};
use sqlx::{Acquire, Error, FromRow, Pool};

use crate::common::builder::BuilderTrait;
use crate::common::database::DatabaseTrait;
use crate::sql::builder::Builder;
use super::connection;
use super::kind::DataKind;

pub struct SqliteQuery;

impl DatabaseTrait for SqliteQuery {
    type Database = sqlx::Sqlite;
    type Row = SqliteRow;
    type QueryResult = SqliteQueryResult;
    type QueryBuilder<'a> = Builder<DataKind<'a>>;

    async fn fetch_one<'a, T>(&self, qb: Self::QueryBuilder<'a>) -> Result<T, Error>
    where
        T: for<'r> FromRow<'r, SqliteRow> + Unpin + Send,
    {
        let pool = self.get_db_pool();
        let (sql, values) = qb.build();
        let mut query = sqlx::query_as::<_, T>(&sql);

        // Bind parameter values to the query
        for value in values {
            query = query.bind(value)
        }

        // Execute the query and return a single record
        query.fetch_one(&*pool).await
    }

    async fn fetch_all<'a, T>(&self, qb: Self::QueryBuilder<'a>) -> Result<Vec<T>, Error>
    where
        T: for<'r> FromRow<'r, SqliteRow> + Unpin + Send,
    {
        let pool = self.get_db_pool();
        let (sql, values) = qb.build();
        let mut query = sqlx::query_as::<_, T>(&sql);

        // Bind parameter values to the query
        for value in values {
            query = query.bind(value)
        }

        // Execute the query and return multiple records
        query.fetch_all(&*pool).await
    }

    async fn fetch_optional<'a, T>(&self, qb: Self::QueryBuilder<'a>) -> Result<Option<T>, Error>
    where
        T: for<'r> FromRow<'r, SqliteRow> + Unpin + Send,
    {
        let pool = self.get_db_pool();
        let (sql, values) = qb.build();
        let mut query = sqlx::query_as::<_, T>(&sql);

        // Bind parameter values to the query
        for value in values {
            query = query.bind(value)
        }

        // Execute the query and return a single optional record
        query.fetch_optional(&*pool).await
    }

    async fn execute<'a>(&self, qb: Self::QueryBuilder<'a>) -> Result<SqliteQueryResult, Error>{
        let pool = self.get_db_pool();
        let mut conn = pool.acquire().await?;
        let mut tx = conn.begin().await?;
        let (sql, values) = qb.build();
        let mut query = sqlx::query(&sql);

        // Bind parameter values to the query
        for value in values {
            query = query.bind(value)
        }

        // Execute the query and handle the transaction
        let result = query.execute(&mut *tx).await;

        match result {
            Ok(r) => {
                // Commit the transaction
                tx.commit().await?;
                Ok(r)
            },
            Err(e) => {
                // Rollback the transaction
                tx.rollback().await?;
                Err(e)
            }
        }
    }

    fn get_db_pool(&self) -> &'static Pool<Self::Database> {
        connection::get_db_pool()
    }
}