use std::marker::PhantomData;
use field_access::FieldAccess;
use sqlx::sqlite::{SqliteQueryResult, SqliteRow};
use sqlx::{Error, FromRow, Sqlite};

use crate::common::builder::BuilderTrait;
use crate::common::database::DatabaseTrait;
use crate::common::operations::{OperationsTrait, CursorPaginatedResult, PaginatedResult};

use super::kind::{is_empty, value_convert, DataKind};
use super::query::SqliteQuery;
use super::sql::{field, QueryBuilder, QueryCondition};
use super::global::{get_global_soft_delete_field, get_global_filter};

/// Data operations structure for performing CRUD operations on entities in the database.
pub struct Operations<'a, T>
where
    T: for<'r> FromRow<'r, SqliteRow> + FieldAccess + Unpin + Send,
{
    /// Table name representing the database table for the entity.
    table_name: &'a str,
    /// Primary key field name used to uniquely identify records in the table, and whether it generates a default value.
    primary_key: (&'a str, bool),
    /// Phantom data for compile-time type checking.
    _phantom: PhantomData<&'a T>,

    query: SqliteQuery,
}

impl<'a, T> OperationsTrait<'a, T, Sqlite> for Operations<'a, T>
where
    T: for<'r> FromRow<'r, SqliteRow> + FieldAccess + Unpin + Send + Sync + Default,
{
    
    type Query = QueryCondition<'a>;    
    type DataKind = DataKind<'a>;
    type QueryResult = SqliteQueryResult;
    fn new(table_name: &'a str, primary_key: (&'a str, bool)) -> Self {
        let primary_key = if primary_key.0.is_empty() {
            (primary_key.0, false)
        } else {
            primary_key
        };
        
        Operations {
            table_name,
            primary_key,
            _phantom: PhantomData,
            query: SqliteQuery,
        }
    }

    async fn insert_one(&self, entity: T) -> Result<Self::QueryResult, Error> {
        let mut cols_names = Vec::new();
        let mut cols_values = Vec::new();

        for (name, field) in entity.fields() {
            if name != self.primary_key.0 || !self.primary_key.1 {
                cols_names.push(name);
                let value = value_convert(field.as_any());
                cols_values.push(value);
            }
        }

        let query = QueryBuilder::insert_into(self.table_name, &cols_names, vec![cols_values]);
        self.query.execute(query).await
    }

    async fn insert_many(&self, entities: Vec<T>) -> Result<Self::QueryResult, Error> {
        let mut cols_names = Vec::new();
        let mut all_cols_values = Vec::new();

        for entity in entities {
            let mut cols_values = Vec::new();
            for (name, field) in entity.fields() {
                if name != self.primary_key.0 || !self.primary_key.1 {
                    if cols_names.is_empty() {
                        cols_names.push(name);
                    }
                    let value = value_convert(field.as_any());
                    cols_values.push(value);
                }
            }
            all_cols_values.push(cols_values);
        }

        let query = QueryBuilder::insert_into(self.table_name, &cols_names, all_cols_values);
        self.query.execute(query).await
    }

    async fn update_one(&self, entity: T, override_empty: bool) -> Result<Self::QueryResult, Error> {
        let mut cols_names = Vec::new();
        let mut cols_values = Vec::new();

        // Part 1: Collect fields to update
        for (name, field) in entity.fields() {
            if name != self.primary_key.0 {
                let value = value_convert(field.as_any());
                if !override_empty && is_empty(&value) {
                    continue;
                }
                cols_names.push(name);
                cols_values.push(value);
            }
        }

        // Part 2: Optimized primary key retrieval
        let primary_key_value = entity.fields()
            .find(|(name, _)| *name == self.primary_key.0)
            .map(|(_, field)| value_convert(field.as_any()))
            .ok_or(Error::RowNotFound)?;

        // Part 3: Build query
        let mut query = QueryBuilder::update(self.table_name, &cols_names, cols_values);
        query.filter(field(self.primary_key.0).eq(primary_key_value.clone()));

        self.query.execute(query).await
    }

    async fn update_many(&self, entities: Vec<T>, override_empty: bool) -> Result<Vec<Self::QueryResult>, Error> {
        let mut results = Vec::new();
        for entity in entities {
            let mut cols_names = Vec::new();
            let mut cols_values = Vec::new();

            for (name, field) in entity.fields() {
                if name != self.primary_key.0 {
                    let value = value_convert(field.as_any());
                    if override_empty && is_empty(&value) {
                        continue;
                    }
                    cols_names.push(name);
                    cols_values.push(value);
                }
            }

            let primary_key_value = {
                let mut primary_key_value = None;
                for (name, field) in entity.fields() {
                    if name == self.primary_key.0 {
                        primary_key_value = Some(value_convert(field.as_any()));
                        break;
                    }
                }
                primary_key_value.ok_or(Error::RowNotFound)?
            };

            let mut query = QueryBuilder::update(self.table_name, &cols_names, cols_values);
            query.filter(field(self.primary_key.0).eq(primary_key_value));
            let result = self.query.execute(query).await?;
            results.push(result);
        }
        Ok(results)
    }

    async fn delete_one(&self, key: impl Into<DataKind<'a>> + Send) -> Result<Self::QueryResult, Error> {
        let key = key.into();

        if let Some((column, exclude_tables)) = get_global_soft_delete_field() {
            if !exclude_tables.contains(&self.table_name) {
                let mut query = QueryBuilder::update(self.table_name, &[column], vec![DataKind::from(true)]);
                query.filter(field(self.primary_key.0).eq(key));
                return self.query.execute(query).await;
            }
        }
        let mut query = QueryBuilder::delete(self.table_name);
        query.filter(field(self.primary_key.0).eq(key));
        self.query.execute(query).await
    }

    async fn delete_many(&self, keys: Vec<impl Into<DataKind<'a>> + Send>) -> Result<Self::QueryResult, Error> {
        let keys: Vec<DataKind<'a>> = keys.into_iter().map(|k| k.into()).collect();
        if let Some((column, exclude_tables)) = get_global_soft_delete_field() {
            if !exclude_tables.contains(&self.table_name) {
                let mut query = QueryBuilder::update(self.table_name, &[column], vec![DataKind::from(true)]);
                query.filter(field(self.primary_key.0).r#in(keys));
                return self.query.execute(query).await;
            }
        }
        let mut query = QueryBuilder::delete(self.table_name);
        query.filter(field(self.primary_key.0).r#in(keys));
        self.query.execute(query).await
    }

    async fn restore_one(&self, key: impl Into<DataKind<'a>> + Send) -> Result<Self::QueryResult, Error> {
        let key = key.into();
        if let Some((column, exclude_tables)) = get_global_soft_delete_field() {
            if !exclude_tables.contains(&self.table_name) {
                let mut query = QueryBuilder::update(self.table_name, &[column], vec![DataKind::from(false)]);
                query.filter(field(self.primary_key.0).eq(key));
                return self.query.execute(query).await;
            }
        }
        Err(Error::Protocol("Restore operation not supported without soft delete configuration".to_string()))
    }

    async fn restore_many(&self, keys: Vec<impl Into<DataKind<'a>> + Send>) -> Result<Self::QueryResult, Error> {
        let keys: Vec<DataKind<'a>> = keys.into_iter().map(|k| k.into()).collect();
        if let Some((column, exclude_tables)) = get_global_soft_delete_field() {
            if !exclude_tables.contains(&self.table_name) {
                let mut query = QueryBuilder::update(self.table_name, &[column], vec![DataKind::from(false)]);
                query.filter(field(self.primary_key.0).r#in(keys));
                return self.query.execute(query).await;
            }
        }
        Err(Error::Protocol("Restore operation not supported without soft delete configuration".to_string()))
    }

    async fn fetch_all(&self, query_condition: Self::Query) -> Result<Vec<T>, Error> {
        let mut builder = QueryBuilder::select(self.table_name, &["*"]);
        query_condition.apply(&mut builder);
        self.apply_global_filters(&mut builder);
        let result = self.query.fetch_all::<T>(builder).await?;
        Ok(result)
    }

    async fn fetch_by_key(&self, id: impl Into<DataKind<'a>> + Send) -> Result<Option<T>, Error> {
        let id = id.into();
        let mut builder = QueryBuilder::select(self.table_name, &["*"]);
        builder.filter(field(self.primary_key.0).eq(id));
        self.apply_global_filters(&mut builder);
        self.query.fetch_optional::<T>(builder).await
    }

    async fn fetch_one(&self, query_condition: Self::Query) -> Result<Option<T>, Error> {
        let mut builder = QueryBuilder::select(self.table_name, &["*"]);
        query_condition.apply(&mut builder);
        self.apply_global_filters(&mut builder);
        self.query.fetch_optional::<T>(builder).await
    }

    async fn fetch_paginated(&self, page_number: u64, page_size: u64, query_condition: Self::Query) -> Result<PaginatedResult<T>, Error> {
        let mut builder = QueryBuilder::select(self.table_name, &["*"]);
        query_condition.apply(&mut builder);       
        self.apply_global_filters(&mut builder);

        let offset = (&page_number - 1) * &page_size;
        builder.limit_offset(page_size, Some(offset));

        let data = self.query.fetch_all::<T>(builder).await?;
        let total = self.count(query_condition).await?;
        Ok(PaginatedResult { data, total, page_number, page_size })
    }

    async fn fetch_by_cursor(&self, limit: u64, query_condition: Self::Query) -> Result<CursorPaginatedResult<T>, Error> 
    where 
        T: Clone,
    {
        let mut builder = QueryBuilder::select(self.table_name, &["*"]);
        query_condition.apply(&mut builder);
        self.apply_global_filters(&mut builder);

        builder.limit_offset(limit,None);
        let data = self.query.fetch_all::<T>(builder).await?;

        // Get the cursor value of the last record
        let next_cursor = data.last().cloned();

        Ok(CursorPaginatedResult {
            data,
            next_cursor,
            page_size: limit,
        })
    }

    async fn exist(&self, query_condition: Self::Query) -> Result<bool, Error> {
        let mut builder = QueryBuilder::select(self.table_name, &["1"]);
        query_condition.apply(&mut builder);
        self.apply_global_filters(&mut builder);
        let result = self.query.fetch_optional::<(i32,)>(builder).await?;
        Ok(result.is_some())
    }

    async fn count(&self, query_condition: Self::Query) -> Result<i64, Error> {
        let mut builder = QueryBuilder::select(self.table_name, &["COUNT(*)"]);
        query_condition.apply(&mut builder);
        self.apply_global_filters(&mut builder);
        let result = self.query.fetch_one::<(i64,)>(builder).await?;
        Ok(result.0)
    }

}

impl<'a, T> Operations<'a, T>
where
    T: for<'r> FromRow<'r, SqliteRow> + FieldAccess + Unpin + Send + Sync + Default,
{
    // Applies global filters including soft delete content filtering
    fn apply_global_filters(&self, builder: &mut QueryBuilder<'a>) {
        if let Some((soft_delete_field, exclude_tables)) = get_global_soft_delete_field() {
            if !exclude_tables.contains(&self.table_name) {
                builder.filter(field(soft_delete_field).eq(false));
            }
        }

        if let Some((filter, exclude_tables)) = get_global_filter() {
            if !exclude_tables.contains(&self.table_name) {
                builder.filter(filter);
            }
        }
    }
}