use std::marker::PhantomData;
use field_access::FieldAccess;
use sqlx::postgres::{PgQueryResult, PgRow};
use sqlx::{Error, FromRow, Postgres};

use crate::common::builder::BuilderTrait;
use crate::common::database::DatabaseTrait;
use crate::common::operations::{OperationsTrait, CursorPaginatedResult, PaginatedResult};
use crate::common::util::check_empty_or_none;

use super::kind::{value_convert, DataKind};
use super::query::PostgresQuery;
use super::sql::{field, QueryBuilder, QueryCondition};
use super::global::{get_global_soft_delete_field, get_global_filter};

/// Data operation structure for performing CRUD operations on database entities.
pub struct Operations<'a, T>
where
    T: for<'r> FromRow<'r, PgRow> + FieldAccess + Unpin + Send,
{
    /// Table name representing the entity's corresponding database table.
    table_name: &'a str,
    /// Primary key field name and whether it auto-generates a default value.
    primary_key: (&'a str, bool),
    /// Phantom data for compile-time type checking.
    _phantom: PhantomData<&'a T>,

    query: PostgresQuery,
}

impl<'a, T> OperationsTrait<'a, T, Postgres> for Operations<'a, T>
where
    T: for<'r> FromRow<'r, PgRow> + FieldAccess + Unpin + Send + Sync + Default,
{
    
    type Query = QueryCondition<'a>;    
    type DataKind = DataKind<'a>;
    type QueryResult = PgQueryResult;
    /// Creates a new instance of `Operations`.
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
            query: PostgresQuery,
        }
    }

    /// Inserts a single entity into the database.
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
        // Execute the insert query
        self.query.execute(query).await
    }

    /// Inserts multiple entities into the database.
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
        // Execute the insert query for multiple entities
        self.query.execute(query).await
    }

    /// Updates a single entity in the database.
    async fn update_one(&self, entity: T, override_empty: bool) -> Result<Self::QueryResult, Error> {
        let mut cols_names = Vec::new();
        let mut cols_values = Vec::new();

        // First part: Collect update fields
        for (name, field) in entity.fields() {
            if name != self.primary_key.0 {
                let value = value_convert(field.as_any());
                if !override_empty && check_empty_or_none(&value) {
                    continue;
                }
                cols_names.push(name);
                cols_values.push(value);
            }
        }

        // Second part: Optimized primary key retrieval
        let primary_key_value = entity.fields()
            .find(|(name, _)| *name == self.primary_key.0)
            .map(|(_, field)| value_convert(field.as_any()))
            .ok_or(Error::RowNotFound)?;

        // Third part: Build the query
        let mut query = QueryBuilder::update(self.table_name, &cols_names, cols_values);
        query.filter(field(self.primary_key.0).eq(primary_key_value.clone()));

        // Execute the update query
        self.query.execute(query).await
    }

    /// Updates multiple entities in the database.
    async fn update_many(&self, entities: Vec<T>, override_empty: bool) -> Result<Vec<Self::QueryResult>, Error> {
        let mut results = Vec::new();
        for entity in entities {
            let mut cols_names = Vec::new();
            let mut cols_values = Vec::new();

            for (name, field) in entity.fields() {
                if name != self.primary_key.0 {
                    let value = value_convert(field.as_any());
                    if override_empty && check_empty_or_none(&value) {
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

    /// Deletes a single entity from the database.
    async fn delete_one(&self, key: impl Into<DataKind<'a>> + Send) -> Result<Self::QueryResult, Error> {
        let key = key.into();

        if let Some((column, exclude_tables)) = get_global_soft_delete_field() {
            if !exclude_tables.contains(&self.table_name) {
                let mut query = QueryBuilder::update(self.table_name, &[column], vec![DataKind::from(true)]);
                query.filter(field(self.primary_key.0).eq(key));
                // Execute the soft delete query
                return self.query.execute(query).await;
            }
        }
        let mut query = QueryBuilder::delete(self.table_name);
        query.filter(field(self.primary_key.0).eq(key));
        // Execute the hard delete query
        self.query.execute(query).await
    }

    /// Deletes multiple entities from the database.
    async fn delete_many(&self, keys: Vec<impl Into<DataKind<'a>> + Send>) -> Result<Self::QueryResult, Error> {
        let keys: Vec<DataKind<'a>> = keys.into_iter().map(|k| k.into()).collect();
        if let Some((column, exclude_tables)) = get_global_soft_delete_field() {
            if !exclude_tables.contains(&self.table_name) {
                let mut query = QueryBuilder::update(self.table_name, &[column], vec![DataKind::from(true)]);
                query.filter(field(self.primary_key.0).r#in(keys));
                // Execute the soft delete query for multiple keys
                return self.query.execute(query).await;
            }
        }
        let mut query = QueryBuilder::delete(self.table_name);
        query.filter(field(self.primary_key.0).r#in(keys));
        // Execute the hard delete query for multiple keys
        self.query.execute(query).await
    }

    /// Restores a single entity in the database.
    async fn restore_one(&self, key: impl Into<DataKind<'a>> + Send) -> Result<Self::QueryResult, Error> {
        let key = key.into();
        if let Some((column, exclude_tables)) = get_global_soft_delete_field() {
            if !exclude_tables.contains(&self.table_name) {
                let mut query = QueryBuilder::update(self.table_name, &[column], vec![DataKind::from(false)]);
                query.filter(field(self.primary_key.0).eq(key));
                // Execute the restore query for a single key
                return self.query.execute(query).await;
            }
        }
        Err(Error::Protocol("Restore operation not supported without soft delete configuration".to_string()))
    }

    /// Restores multiple entities in the database.
    async fn restore_many(&self, keys: Vec<impl Into<DataKind<'a>> + Send>) -> Result<Self::QueryResult, Error> {
        let keys: Vec<DataKind<'a>> = keys.into_iter().map(|k| k.into()).collect();
        if let Some((column, exclude_tables)) = get_global_soft_delete_field() { // 修复拼写错误
            if !exclude_tables.contains(&self.table_name) {
                let mut query = QueryBuilder::update(self.table_name, &[column], vec![DataKind::from(false)]);
                query.filter(field(self.primary_key.0).r#in(keys));
                // Execute the restore query for multiple keys
                return self.query.execute(query).await;
            }
        }
        Err(Error::Protocol("Restore operation not supported without soft delete configuration".to_string()))
    }

    /// Fetches all entities that match the query condition.
    async fn fetch_all(&self, query_condition: Self::Query) -> Result<Vec<T>, Error> {
        let mut builder = QueryBuilder::select(self.table_name, &["*"]);
        query_condition.apply(&mut builder);
        // Apply soft delete filter if necessary
        self.apply_global_filters(&mut builder);
        let result = self.query.fetch_all::<T>(builder).await?;
        Ok(result)
    }

    /// Fetches an entity by its key.
    async fn fetch_by_key(&self, id: impl Into<DataKind<'a>> + Send) -> Result<Option<T>, Error> {
        let id = id.into();
        let mut builder = QueryBuilder::select(self.table_name, &["*"]);
        builder.filter(field(self.primary_key.0).eq(id));
        // Apply soft delete filter if necessary
        self.apply_global_filters(&mut builder);
        // Fetch an optional record by key
        self.query.fetch_optional::<T>(builder).await
    }

    /// Fetches a single entity that match the query condition.
    async fn fetch_one(&self, query_condition: Self::Query) -> Result<Option<T>, Error> {
        let mut builder = QueryBuilder::select(self.table_name, &["*"]);
        query_condition.apply(&mut builder);
        // Apply soft delete filter if necessary
        self.apply_global_filters(&mut builder);
        // Fetch an optional record based on the query condition
        self.query.fetch_optional::<T>(builder).await
    }

    /// Fetches paginated entities that match the query condition.
    async fn fetch_paginated(&self, page_number: u64, page_size: u64, query_condition: Self::Query) -> Result<PaginatedResult<T>, Error> {
        let mut builder = QueryBuilder::select(self.table_name, &["*"]);
        query_condition.apply(&mut builder);       
        // Apply soft delete filter if necessary
        self.apply_global_filters(&mut builder);

        let offset = (&page_number - 1) * &page_size;
        builder.limit_offset(page_size, Some(offset));

        let data = self.query.fetch_all::<T>(builder).await?;
        let total = self.count(query_condition).await?;
        Ok(PaginatedResult { data, total, page_number, page_size })
    }

    /// Fetches entities by cursor that match the query condition.
    async fn fetch_by_cursor(&self, limit: u64, query_condition: Self::Query) -> Result<CursorPaginatedResult<T>, Error> 
    where 
        T: Clone,
    {
        let mut builder = QueryBuilder::select(self.table_name, &["*"]);
        query_condition.apply(&mut builder);
        // Apply soft delete filter if necessary
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

    /// Checks if an entity exists that match the query condition.
    async fn exist(&self, query_condition: Self::Query) -> Result<bool, Error> {
        let mut builder = QueryBuilder::select(self.table_name, &["1"]);
        query_condition.apply(&mut builder);
        // Apply soft delete filter if necessary
        self.apply_global_filters(&mut builder);
        let result = self.query.fetch_optional::<(i32,)>(builder).await?;
        Ok(result.is_some())
    }

    /// Counts the number of entities that match the query condition.
    async fn count(&self, query_condition: Self::Query) -> Result<i64, Error> {
        let mut builder = QueryBuilder::select(self.table_name, &["COUNT(*)"]);
        query_condition.apply(&mut builder);
        // Apply soft delete filter if necessary
        self.apply_global_filters(&mut builder);
        let result = self.query.fetch_one::<(i64,)>(builder).await?;
        Ok(result.0)
    }

}

impl<'a, T> Operations<'a, T>
where
    T: for<'r> FromRow<'r, PgRow> + FieldAccess + Unpin + Send + Sync + Default,
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