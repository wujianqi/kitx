use std::borrow::Cow;
use std::marker::PhantomData;
use std::sync::Arc;
use field_access::FieldAccess;
use sqlx::mysql::{MySqlQueryResult, MySqlRow};
use sqlx::{Error, FromRow, MySql};

use crate::common::query::QueryExecutor;
use crate::builders::composite::CompositeKeyTable;
use crate::common::types::{CursorPaginatedResult, PaginatedResult};
use crate::utils::query_condition::QueryCondition;

use super::kind::DataKind;
use super::query::MySqlQuery;
use super::{Delete, Select};
use super::global::{get_global_soft_delete_field, get_global_filter};

/// Data operations structure for composite primary key scenarios
pub struct MutliKeyOperations<'a, T>
where
    T: for<'r> FromRow<'r, MySqlRow> + FieldAccess + Default + Unpin + Send + Sync,
{
    table_query: CompositeKeyTable<'a, T, DataKind<'a>, MySql, DataKind<'a>>,
    query: Arc<MySqlQuery<'a>>,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T> MutliKeyOperations<'a, T>
where
    T: for<'r> FromRow<'r, MySqlRow> + FieldAccess + Unpin + Send + Sync + Default,
{
    /// Create a new Relations instance with composite primary keys
    pub fn new(table_name: &'a str, primarys: Vec<&'a str>) -> Self {
        let table_query = CompositeKeyTable::new(
            table_name,
            primarys,
            get_global_soft_delete_field(),
            get_global_filter(),
        );

        MutliKeyOperations { 
            table_query, 
            query: Arc::new(MySqlQuery::new()), 
            _phantom: PhantomData 
        }
    }

    pub fn set(mut self, query: Arc<MySqlQuery<'a>>) -> Self {
        self.query = query;
        self
    }

    // Composite key specific operations
    pub async fn get_one_by_keys(&self, keys: &[(&str, DataKind<'a>)]) -> Result<Option<T>, Error> {
        let builder = self.table_query.get_one_by_keys(keys)?;
        self.query.fetch_optional::<T, Select>(builder).await
    }

    pub async fn get_one<F>(&self, query_condition: F) -> Result<Option<T>, Error>
    where
        F: Fn(&mut Select<'a>) + Send + Sync + 'a,
    {
        let builder = self.table_query.get_one(query_condition);
        self.query.fetch_optional::<T, Select>(builder).await
    }

    pub async fn get_list<F>(&self, query_condition: F) -> Result<Vec<T>, Error>
    where
        F: Fn(&mut Select<'a>) + Send + Sync + 'a,
    {
        let builder = self.table_query.get_list(query_condition);
        self.query.fetch_all::<T, Select>(builder).await
    }

    pub async fn get_list_paginated<F>(
        &self,
        page_number: u64,
        page_size: u64,
        query_condition: F,
    ) -> Result<PaginatedResult<T>, Error>
    where
        F: Fn(&mut Select<'a>) + Send + Sync + 'a,
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

    pub async fn get_list_by_cursor<F>(
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

    pub async fn insert_one(&self, entity: T) -> Result<MySqlQueryResult, Error> {
        let builder = self.table_query.insert_one(entity)?;
        self.query.execute(builder).await
    }

    pub async fn insert_many(&self, entities: Vec<T>) -> Result<MySqlQueryResult, Error> {
        let builder = self.table_query.insert_many(entities)?;
        self.query.execute(builder).await
    }

    pub async fn delete_by_keys(&self, keys: &[(&str, DataKind<'a>)]) -> Result<MySqlQueryResult, Error> {
        if self.table_query.is_soft_delete_enabled() {
            let builder = self.table_query.soft_delete_by_keys(&keys)?;
            self.query.execute(builder).await
        } else {
            let builder = self.table_query.delete_by_keys(&keys)?;
            self.query.execute(builder).await
        }
    }

    /// Delete records based on custom condition configuration
    pub async fn delete_by_cond<F>(&self, query_condition: F) -> Result<MySqlQueryResult, Error>
    where
        F: Fn(&mut Delete<'a>) + Send + Sync + 'a,
    {
        if self.table_query.is_soft_delete_enabled() {
            let builder = self.table_query.soft_delete_by_cond(query_condition)?;
            self.query.execute(builder).await
        } else {
            let builder = self.table_query.delete_by_cond(query_condition);
            self.query.execute(builder).await
        }
    }

    pub async fn restore_one(&self, key: impl Into<DataKind<'a>> + Send) -> Result<MySqlQueryResult, Error> {
        let builder = self.table_query.restore_one(key)?;
        self.query.execute(builder).await
    }

    pub async fn restore_many(&self, keys: Vec<impl Into<DataKind<'a>> + Send>) -> Result<MySqlQueryResult, Error> {
        let data_keys: Vec<_> = keys.into_iter().map(|k| ("id", k.into())).collect();
        let builder = self.table_query.restore_by_keys(&data_keys)?;
        self.query.execute(builder).await
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