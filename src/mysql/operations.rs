use std::marker::PhantomData;
use field_access::FieldAccess;
use sqlx::mysql::{MySqlQueryResult, MySqlRow};
use sqlx::{Error, FromRow, MySql};

use crate::common::builder::BuilderTrait;
use crate::common::database::DatabaseTrait;
use crate::common::operations::{OperationsTrait, CursorPaginatedResult, PaginatedResult};

use super::kind::{is_empty, value_convert, DataKind};
use super::query::MySqlQuery;
use super::sql::{field, QueryBuilder, QueryCondition};

/// Data operation structure for performing CRUD operations on database entities.
pub struct Operations<'a, T>
where
    T: for<'r> FromRow<'r, MySqlRow> + FieldAccess + Unpin + Send,
{
    /// Table name representing the entity's corresponding database table.
    table_name: &'a str,
    /// Primary key field name and whether it auto-generates a default value.
    primary_key: (&'a str, bool),
    /// Soft delete field name and filter flag for marking records as deleted and filtering.
    soft_delete_info: Option<(&'a str, bool)>,
    /// Phantom data for compile-time type checking.
    _phantom: PhantomData<&'a T>,

    query: MySqlQuery,
}

impl<'a, T> OperationsTrait<'a, T, MySql> for Operations<'a, T>
where
    T: for<'r> FromRow<'r, MySqlRow> + FieldAccess + Unpin + Send + Sync + Default,
{
    
    type Query = QueryCondition<'a>;    
    type DataKind = DataKind<'a>;
    type QueryResult = MySqlQueryResult;
    /// Creates a new instance of `Operations`.
    fn new(table_name: &'a str, primary_key: (&'a str, bool), soft_delete_info: Option<(&'a str, bool)>) -> Self {
        let primary_key = if primary_key.0.is_empty() {
            (primary_key.0, false)
        } else {
            primary_key
        };

        Operations {
            table_name,
            primary_key,
            soft_delete_info,
            _phantom: PhantomData,
            query: MySqlQuery,
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
                if !override_empty && is_empty(&value) {
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

    /// Deletes a single entity from the database.
    async fn delete_one(&self, key: impl Into<DataKind<'a>> + Send) -> Result<Self::QueryResult, Error> {
        let key = key.into();

        if let Some((column, _)) = self.soft_delete_info {
            let mut query = QueryBuilder::update(self.table_name, &[column], vec![DataKind::from(true)]);
            query.filter(field(self.primary_key.0).eq(key));
            // Execute the soft delete query
            self.query.execute(query).await
        } else {
            let mut query = QueryBuilder::delete(self.table_name);
            query.filter(field(self.primary_key.0).eq(key));
            // Execute the hard delete query
            self.query.execute(query).await
        }
    }

    /// Deletes multiple entities from the database.
    async fn delete_many(&self, keys: Vec<impl Into<DataKind<'a>> + Send>) -> Result<Self::QueryResult, Error> {
        let keys: Vec<DataKind<'a>> = keys.into_iter().map(|k| k.into()).collect();
        if let Some((column, _)) = self.soft_delete_info {
            let mut query = QueryBuilder::update(self.table_name, &[column], vec![DataKind::from(true)]);
            query.filter(field(self.primary_key.0).r#in(keys));
            // Execute the soft delete query for multiple keys
            self.query.execute(query).await
        } else {
            let mut query = QueryBuilder::delete(self.table_name);
            query.filter(field(self.primary_key.0).r#in(keys));
            // Execute the hard delete query for multiple keys
            self.query.execute(query).await
        }
    }

    /// Restores a single entity in the database.
    async fn restore_one(&self, key: impl Into<DataKind<'a>> + Send) -> Result<Self::QueryResult, Error> {
        let key = key.into();
        let mut query = QueryBuilder::update(self.table_name, &[self.soft_delete_info.as_ref().unwrap().0], vec![DataKind::from(false)]);
        query.filter(field(self.primary_key.0).eq(key));
        // Execute the restore query for a single key
        self.query.execute(query).await
    }

    /// Restores multiple entities in the database.
    async fn restore_many(&self, keys: Vec<impl Into<DataKind<'a>> + Send>) -> Result<Self::QueryResult, Error> {
        let keys: Vec<DataKind<'a>> = keys.into_iter().map(|k| k.into()).collect();
        let mut query = QueryBuilder::update(self.table_name, &[self.soft_delete_info.as_ref().unwrap().0], vec![DataKind::from(false)]);
        query.filter(field(self.primary_key.0).r#in(keys));
        // Execute the restore query for multiple keys
        self.query.execute(query).await
    }

    /// Fetches all entities that match the query condition.
    async fn fetch_all(&self, query_condition: Self::Query) -> Result<Vec<T>, Error> {
        let mut builder = QueryBuilder::select(self.table_name, &["*"]);
        query_condition.apply(&mut builder);
        // Apply soft delete filter if necessary
        self.apply_soft_delete_filter(&mut builder);
        let result = self.query.fetch_all::<T>(builder).await?;
        Ok(result)
    }

    /// Fetches an entity by its key.
    async fn fetch_by_key(&self, id: impl Into<DataKind<'a>> + Send) -> Result<Option<T>, Error> {
        let id = id.into();
        let mut builder = QueryBuilder::select(self.table_name, &["*"]);
        builder.filter(field(self.primary_key.0).eq(id));
        // Apply soft delete filter if necessary
        self.apply_soft_delete_filter(&mut builder);
        // Fetch an optional record by key
        self.query.fetch_optional::<T>(builder).await
    }

    /// Fetches a single entity that matches the query condition.
    async fn fetch_one(&self, query_condition: Self::Query) -> Result<Option<T>, Error> {
        let mut builder = QueryBuilder::select(self.table_name, &["*"]);
        query_condition.apply(&mut builder);
        // Apply soft delete filter if necessary
        self.apply_soft_delete_filter(&mut builder);
        // Fetch an optional record based on the query condition
        self.query.fetch_optional::<T>(builder).await
    }

    /// Fetches paginated entities that match the query condition.
    async fn fetch_paginated(&self, page_number: u64, page_size: u64, query_condition: Self::Query) -> Result<PaginatedResult<T>, Error> {
        let mut builder = QueryBuilder::select(self.table_name, &["*"]);
        query_condition.apply(&mut builder);       
        // Apply soft delete filter if necessary
        self.apply_soft_delete_filter(&mut builder);

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
        self.apply_soft_delete_filter(&mut builder);

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

    /// Checks if an entity exists that matches the query condition.
    async fn exist(&self, query_condition: Self::Query) -> Result<bool, Error> {
        let mut builder = QueryBuilder::select(self.table_name, &["1"]);
        query_condition.apply(&mut builder);
        // Apply soft delete filter if necessary
        self.apply_soft_delete_filter(&mut builder);
        let result = self.query.fetch_optional::<(i32,)>(builder).await?;
        Ok(result.is_some())
    }

    /// Counts the number of entities that match the query condition.
    async fn count(&self, query_condition: Self::Query) -> Result<i64, Error> {
        let mut builder = QueryBuilder::select(self.table_name, &["COUNT(*)"]);
        query_condition.apply(&mut builder);
        // Apply soft delete filter if necessary
        self.apply_soft_delete_filter(&mut builder);
        let result = self.query.fetch_one::<(i64,)>(builder).await?;
        Ok(result.0)
    }

}

impl<'a, T> Operations<'a, T>
where
    T: for<'r> FromRow<'r, MySqlRow> + FieldAccess + Unpin + Send + Sync + Default,
{
    // Applies soft delete content filtering
    fn apply_soft_delete_filter(&self, builder: &mut QueryBuilder<'a>) {
        if let Some((soft_delete_field, filter_soft_deleted)) = &self.soft_delete_info {
            if *filter_soft_deleted {
                builder.filter(field(soft_delete_field).eq(false));
            }
        }
    }
}
