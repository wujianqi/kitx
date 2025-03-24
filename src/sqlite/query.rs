use sqlx::sqlite::{SqliteQueryResult, SqliteRow};
use sqlx::{Acquire, Encode, Error, FromRow, Pool, Sqlite, Type};

use crate::common::builder::BuilderTrait;
use crate::common::database::DatabaseTrait;
use crate::common::error::OperationError;
use crate::sql::select::SelectBuilder;
use super::connection;
use super::kind::DataKind;

pub struct SqliteQuery;

impl DatabaseTrait for SqliteQuery {
    type Database = Sqlite;
    type Row = SqliteRow;
    type QueryResult = SqliteQueryResult;
    type SelectQuery<'a> = SelectBuilder<DataKind<'a>>;

    async fn fetch_one<'a, T>(&self, qb: Self::SelectQuery<'a>) -> Result<T, Error>
    where
        T: for<'r> FromRow<'r, SqliteRow> + Unpin + Send + 'a,
    {
        let (sql, values) = qb.build();
        if values.is_empty() {
            return Err(OperationError::new("No parameters provided".to_string()));
        }
        let pool = self.get_db_pool();
        let mut query = sqlx::query_as::<_, T>(&sql);

        // Bind parameter values to the query
        for value in values {
            query = query.bind(value)
        }

        // Execute the query and return a single record
        query.fetch_one(&*pool).await
    }

    async fn fetch_all<'a, T>(&self, qb: Self::SelectQuery<'a>) -> Result<Vec<T>, Error>
    where
        T: for<'r> FromRow<'r, SqliteRow> + Unpin + Send + 'a,
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

    async fn fetch_optional<'a, T>(&self, qb: Self::SelectQuery<'a>) -> Result<Option<T>, Error>
    where
        T: for<'r> FromRow<'r, SqliteRow> + Unpin + Send + 'a,
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

    async fn execute<'a, B, T>(&self, qb: B) -> Result<Self::QueryResult, Error>
    where
        B: BuilderTrait<T> + Send,
        T: Send + Unpin + Encode<'a, Self::Database> + Type<Self::Database> + 'a,
    {
        let (sql, values) = qb.build();
        if values.is_empty() {
            return Err(OperationError::new("No parameters provided".to_string()));
        }
        let pool = self.get_db_pool();
        let mut conn = pool.acquire().await?;
        let mut tx = conn.begin().await?;
        
        let sql_str: &str = Box::leak(sql.into_boxed_str());
        let mut query = sqlx::query(sql_str);

        // Bind parameter values to the query
        for value in values {
            query = query.bind(value);
        }
        //dbg!(&sql_str);

        // Execute the query and handle the transaction
        let result = query.execute(&mut *tx).await;

        match result {
            Ok(r) => {
                // Commit the transaction
                tx.commit().await?;
                Ok(r)
            }
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
