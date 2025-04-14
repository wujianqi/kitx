use std::borrow::Cow;
use std::marker::PhantomData;
use field_access::FieldAccess;
use sqlx::sqlite::{SqliteQueryResult, SqliteRow};
use sqlx::{Error, FromRow, Sqlite};

use crate::common::query::QueryExecutor;
use crate::common::operations::{OperationsTrait, CursorPaginatedResult, PaginatedResult};
use crate::builders::base::TableQueryBuilder;
use crate::utils::query::QueryCondition;

use super::kind::DataKind;
use super::query::SqliteQuery;
use super::sql::{Delete, Select, Update};
use super::global::{get_global_soft_delete_field, get_global_filter};


/// Data operations structure for performing CRUD operations on entities in the database.
pub struct Operations<'a, T>
where
    T: for<'r> FromRow<'r, SqliteRow> + FieldAccess + Default + Unpin + Send + Sync,
{
    
    table_query: TableQueryBuilder<'a, T, DataKind<'a>, Sqlite, DataKind<'a>> ,
    query: SqliteQuery,

    /// Phantom data for compile-time type checking.
    _phantom: PhantomData<&'a T>,    
}

impl<'a, T> OperationsTrait<'a, T, Sqlite, DataKind<'a>> for Operations<'a, T>
where
    T: for<'r> FromRow<'r, SqliteRow> + FieldAccess + Unpin + Send + Sync + Default,
{
    type QueryFilter<'b> = Select<'a>;
    type DeleteFilter<'b> = Delete<'a>;
    type UpdateFilter<'b> = Update<'a>;

    /// Create a new Operations instance.
    /// # Arguments
    /// * `table_name` - Table name representing the database table for the entity.
    /// * `primary_key` - Primary key field name used to uniquely identify records in the table, and whether it generates a default value.
    /// 
    fn new(table_name: &'a str, primary_key: (&'a str, bool)) -> Self {
        let primary_key = if primary_key.0.is_empty() {
            (primary_key.0, false)
        } else {
            primary_key
        };

        let table_query = TableQueryBuilder::new(
            table_name,
            primary_key,
            get_global_soft_delete_field(),
            get_global_filter(),
        );        
        Operations { table_query,  query: SqliteQuery, _phantom: PhantomData}
    }

    async fn get_list<F>(&self, query_condition: F) -> Result<Vec<T>, Error>
    where
        F: Fn(&mut Select<'a>) + Send + Sync + 'a,
    {
        let builder = self.table_query.get_list(query_condition);
        self.query.fetch_all::<T, Select>(builder).await
    }

    async fn get_by_key(&self, id: impl Into<DataKind<'a>> + Send) -> Result<Option<T>, Error> {
        let builder = self.table_query.get_by_key(id);
        self.query.fetch_optional::<T, Select>(builder).await
    }

    async fn get_one<F>(&self, query_condition: F) -> Result<Option<T>, Error>
    where
        F: Fn(&mut Select<'a>) + Send + Sync + 'a,
    {
        let builder = self.table_query.get_one(query_condition);
        self.query.fetch_optional::<T, Select>(builder).await
    }

    async fn get_list_paginated<F>(
        &self,
        page_number: u64,
        page_size: u64,
        query_condition: F,
    ) -> Result<PaginatedResult<T>, Error>
    where
        F: Fn(&mut Select<'a>) + Send + Sync +'a,
    {
        let qc = QueryCondition::new(query_condition);
        
        let builder = self.table_query.get_list_paginated(page_number, page_size, qc.get())?;
        
        let (total, data) = tokio::join!(
            self.count(qc.get()),
            self.query.fetch_all::<T, Select>(builder)
        );

        Ok(PaginatedResult {
            data: data?,
            total: total?,
            page_number,
            page_size,
        })
    }

    async fn get_list_by_cursor<F>(
        &self,
        limit: u64,
        query_condition: F,
    ) -> Result<CursorPaginatedResult<T>, Error>
    where
        F: Fn(&mut Select<'a>) + Send + Sync  + 'a,
        T: Clone,
    {
        let builder = self.table_query.get_list_by_cursor(limit, query_condition)?;
        let data = self.query.fetch_all::<T, Select>(builder).await?;
        let next_cursor = data.last().map(|item| Cow::Borrowed(item).into_owned());
        Ok(CursorPaginatedResult {
            data,
            next_cursor,
            page_size: limit,
        })
    }

    async fn insert_one(&self, entity: T) -> Result<SqliteQueryResult, Error> {
        let builder =  self.table_query.insert_one(entity)?;
        self.query.execute(builder).await
    }

    async fn insert_many(&self, entities: Vec<T>) -> Result<SqliteQueryResult, Error> {
        let builder = self.table_query.insert_many(entities)?;
        self.query.execute(builder).await
    }

    async fn update_by_key(&self, entity: T) -> Result<SqliteQueryResult, Error> {
        let builder = self.table_query.update_by_key(entity)?;
        self.query.execute(builder).await
    }

    async fn update_one<F>(&self, entity: T, query_condition: F) -> Result<SqliteQueryResult, Error>
    where
        F: Fn(&mut Self::UpdateFilter<'a>) + Send + Sync + 'a,
    {
        let builder = self.table_query.update_one(entity, query_condition)?;
        self.query.execute(builder).await
    }

    async fn upsert_one(&self, entity: T) -> Result<SqliteQueryResult, Error> {
        let builder = self.table_query.upsert_one(entity)?;
        self.query.execute(builder).await
    }

    async fn upsert_many(&self, entities: Vec<T>) -> Result<SqliteQueryResult, Error> {
        let builder = self.table_query.upsert_many(entities)?;
        self.query.execute(builder).await
    }

    async fn delete_by_key(&self, key: impl Into<DataKind<'a>> + Send) -> Result<SqliteQueryResult, Error> {
        let builder = self.table_query.delete_by_key(key);
        self.query.execute(builder).await
    }

    async fn delete_many(&self, keys: Vec<impl Into<DataKind<'a>>>) -> Result<SqliteQueryResult, Error> {
        let builder = self.table_query.delete_many(keys)?;
        self.query.execute(builder).await
    }

    async fn delete_by_cond<F>(&self, query_condition: F) -> Result<SqliteQueryResult, Error>
    where
        F: Fn(&mut Self::DeleteFilter<'a>) + Send + Sync + 'a,
    {
        let builder = self.table_query.delete_by_cond(query_condition);
        self.query.execute(builder).await
    }

    async fn restore_one(&self, key: impl Into<DataKind<'a>> + Send) -> Result<SqliteQueryResult, Error> {
        let builder = self.table_query.restore_one(key)?;
        self.query.execute(builder).await
    }

    async fn restore_many(&self, keys: Vec<impl Into<DataKind<'a>> + Send>) -> Result<SqliteQueryResult, Error> {
        let builder = self.table_query.restore_many(keys)?;
        self.query.execute(builder).await
    }

    async fn exist<F>(&self, query_condition: F) -> Result<bool, Error>
    where
        F: Fn(&mut Select<'a>) + Send + Sync + 'a,
    {
        let builder = self.table_query.exist(query_condition);
        let result = self.query.fetch_optional::<(i32,), Select>(builder).await?;
        Ok(result.is_some())
    }

    async fn count<F>(&self, query_condition: F) -> Result<u64, Error>
    where
        F: Fn(&mut Select<'a>) + Send + Sync + 'a,
    {
        let builder = self.table_query.count(query_condition);
        let result = self.query.fetch_one::<(u64,), Select>(builder).await?;
        Ok(result.0)
    }
}
