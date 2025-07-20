use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;
use field_access::FieldAccess;
use sqlx::mysql::{MySqlQueryResult, MySqlRow};
use sqlx::{Error, FromRow, MySql};

use crate::common::builder::FilterTrait;
use crate::common::query::QueryExecutor;
use crate::common::operations::{OpsBuilderTrait, OpsActionTrait};
use crate::builders::composite::CompositeKeyTable;
use crate::common::types::{CursorPaginatedResult, PaginatedResult, PrimaryKey};
use crate::utils::query_condition::QueryCondition;

use super::kind::DataKind;
use super::query::MySqlQuery;
use super::{Delete, Select, Update};
use super::global::{get_global_soft_delete_field, get_global_filter};

/// Data operations structure for performing CRUD operations on entities in the database.
pub struct Operations<'a, T>
where
    T: for<'r> FromRow<'r, MySqlRow> + FieldAccess + Default + Clone + Debug + Unpin + Send + Sync,
{
    
    table_query: CompositeKeyTable<'a, T, DataKind<'a>, MySql, DataKind<'a>> ,
    query: Arc<MySqlQuery<'a>>,
    _phantom: PhantomData<&'a T>,
}


impl<'a, T> Operations<'a, T>
where
    T: for<'r> FromRow<'r, MySqlRow> + FieldAccess + Default + Clone + Debug  + Unpin + Send + Sync,
{
    
    
    /// Create a new Operations instance.
    /// # Arguments
    /// * `table_name` - Table name representing the database table for the entity.
    /// * `primarys` - Primary keys for the entity.
    pub fn new(table_name: &'a str, primarys: Vec<&'a str>) -> Self 
    {
        let table_query = CompositeKeyTable::new(
            table_name,
            primarys,
            get_global_soft_delete_field(),
            get_global_filter(),
        );

        Operations { 
            table_query, 
            query: Arc::new(MySqlQuery::new()), 
            _phantom: PhantomData 
        }
    }
    
    /// Sets the query for the operations.
    pub fn set(mut self, query: Arc<MySqlQuery<'a>>) -> Self {
        self.query = query;
        self
    }
}

impl<'a, T> OpsActionTrait<'a, T, MySql, DataKind<'a>> for Operations<'a, T>
where
    T: for<'r> FromRow<'r, MySqlRow> + FieldAccess + Unpin + Send + Sync + Default + Clone + Debug,
{
    type QueryFilter<'b> = Select<'a>;
    type UpdateFilter<'b> = Update<'a>;
    type DeleteFilter<'b> = Delete<'a>;

    async fn insert_one(&self, entity: T) -> Result<MySqlQueryResult, Error> {
        let builder =  self.table_query.insert_many(vec![entity])?;
        self.query.execute(builder).await
    }

    async fn insert_many(&self, entities: Vec<T>) -> Result<MySqlQueryResult, Error> {
        let builder = self.table_query.insert_many(entities)?;
        self.query.execute(builder).await
    }

    async fn update_one(&self, entity: T) -> Result<MySqlQueryResult, Error>
    {
        let builder = self.table_query.update_one(entity)?;
        self.query.execute(builder).await
    }

    async fn update_by_cond<F>(&self, query_condition: F) -> Result<MySqlQueryResult, Error>
    where
        F: Fn(&mut Self::UpdateFilter<'a>) + Send + Sync,
    {
        let builder = self.table_query.update_by_cond(query_condition)?;
        self.query.execute(builder).await
    }

    async fn upsert_one(&self, entity: T) -> Result<MySqlQueryResult, Error> {
        self.upsert_many(vec![entity]).await
    }

    async fn upsert_many(&self, entities: Vec<T>) -> Result<MySqlQueryResult, Error> {
        let (mut builder, cols, _) = self.table_query.upsert_many(entities, false)?;
        builder = builder.on_duplicate(&cols, None);
        self.query.execute(builder).await
    }

    async fn delete_by_pk(&self, key: impl Into<PrimaryKey<DataKind<'a>>> + Send + Sync) -> Result<MySqlQueryResult, Error> {
        if self.table_query.is_soft_delete_enabled() {
            let builder = self.table_query.soft_delete_by_pk(key)?;
            self.query.execute(builder).await
        } else {
            let builder = self.table_query.delete_by_pk(key)?;
            self.query.execute(builder).await
        }
    }

    async fn delete_by_cond<F>(&self, query_condition: F) -> Result<MySqlQueryResult, Error>
    where
        F: Fn(&mut Delete<'a>) + Send + Sync,
    {
        if self.table_query.is_soft_delete_enabled() {
            let builder = self.table_query.soft_delete_by_cond(|b:  &mut Update<'a>|{
                let mut delete_builder = Delete::default();
                query_condition(&mut delete_builder);

                for condition in delete_builder.take_where_clauses() {
                    b.and_where_mut(condition);
                }
            })?;
            self.query.execute(builder).await
        } else {
            let builder = self.table_query.delete_by_cond(query_condition)?;
            self.query.execute(builder).await
        }
    }

    async fn exists<F>(&self, query_condition: F) -> Result<bool, Error>
    where
        F: Fn(&mut Select<'a>) + Send + Sync,
    {
        let builder = self.table_query.exists(query_condition);
        let result = self.query.fetch_optional::<(i32,), Select>(builder).await?;
        Ok(result.is_some())
    }

    async fn count<F>(&self, query_condition: F) -> Result<u64, Error>
    where
        F: Fn(&mut Select<'a>) + Send + Sync,
    {
        let builder = self.table_query.count(query_condition);
        let result = self.query.fetch_one::<(u64,), Select>(builder).await?;
        Ok(result.0)
    }
    
    
    async fn get_one_by_pk(&self, key: impl Into<PrimaryKey<DataKind<'a>>> + Send + Sync) -> Result<Option<T>, Error> {
        let builder = self.table_query.fetch_by_pk(key)?;
        self.query.fetch_optional::<T, Select>(builder).await
    }
    
    async fn get_one_by_cond<F>(&self, query_condition: F) -> Result<Option<T>, Error>
    where
        F: Fn(&mut Select<'a>) + Send + Sync,
    {
        let builder = self.table_query.fetch_by_cond(query_condition);
        self.query.fetch_optional::<T, Select>(builder).await
    }
    
    async fn get_list_by_cond<F>(&self, query_condition: F) -> Result<Vec<T>, Error>
    where
        F: Fn(&mut Select<'a>) + Send + Sync,
    {
        let builder = self.table_query.fetch_by_cond(query_condition);
        self.query.fetch_all::<T, Select>(builder).await
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

    async fn get_list_by_cursor<F, C>(
        &self,
        limit: u64,
        query_condition: F,
        cursor_extractor: impl Fn(&T) -> C + Send + Sync,
    ) -> Result<CursorPaginatedResult<T, C>, Error>
    where
        F: Fn(&mut Select<'a>) + Send + Sync + 'a,
        C: Send + Sync,
    {
        let builder = self.table_query.get_list_by_cursor(limit, query_condition)?;
        let data = self.query.fetch_all::<T, _>(builder).await?;
        let next_cursor = data.last().map(&cursor_extractor);

        Ok(CursorPaginatedResult {
            data,
            next_cursor,
            limit,
        })
    }

    async fn restore_by_pk(&self, key: impl Into<PrimaryKey<DataKind<'a>>> + Send + Sync) -> Result<MySqlQueryResult, Error> {
        let builder = self.table_query.restore_by_pk(key)?;
        self.query.execute(builder).await
    }

    async fn restore_by_cond<F>(&self, query_condition: F) -> Result<MySqlQueryResult, Error>
    where
        F: Fn(&mut Self::UpdateFilter<'a>) + Send + Sync,
    {
        let builder = self.table_query.restore_by_cond(query_condition)?;
        self.query.execute(builder).await
    }

}
