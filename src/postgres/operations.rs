use std::marker::PhantomData;
use field_access::FieldAccess;
use sqlx::postgres::{PgQueryResult, PgRow};
use sqlx::{Error, FromRow, Postgres};

use crate::common::builder::FilterTrait;
use crate::common::database::DatabaseTrait;
use crate::common::error::OperationError;
use crate::common::operations::{OperationsTrait, CursorPaginatedResult, PaginatedResult};
use crate::sql::filter::Expr;

use super::kind::{value_convert, DataKind};
use super::query::PostgresQuery;
use super::sql::{col, Delete, Insert, Select, Update};
use super::global::{get_global_soft_delete_field, get_global_filter};

/// Data operations structure for performing CRUD operations on entities in the database.
pub struct Operations<'a, T>
where
    T: for<'r> FromRow<'r, PgRow> + FieldAccess + Unpin + Send,
{
    /// Table name representing the database table for the entity.
    table_name: &'a str,
    /// Primary key field name used to uniquely identify records in the table, and whether it generates a default value.
    primary_key: (&'a str, bool),
    /// Phantom data for compile-time type checking.
    _phantom: PhantomData<&'a T>,

    query: PostgresQuery,
}

impl<'a, T> OperationsTrait<'a, T, Postgres> for Operations<'a, T>
where
    T: for<'r> FromRow<'r, PgRow> + FieldAccess + Unpin + Send + Sync + Default,
{
    type DataType = DataKind<'a>;
    type QueryResult = PgQueryResult;
    type QueryFilter<'b> = Select<'a>;
    type DeleteFilter<'b> = Delete<'a>;
    type UpdateFilter<'b> = Update<'a>;

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

    async fn get_list<F>(&self, query_condition: Option<F>) -> Result<Vec<T>, Error> 
    where
        F: FnOnce(&mut Self::QueryFilter<'a>) + Send + 'a
    {
        let mut builder = Select::columns(&["*"]).from(self.table_name);
        self.apply_global_filters(&mut builder);
        if let Some(condition) = query_condition {
            condition(&mut builder);
        }
        self.query.fetch_all::<T>(builder).await
    }

    async fn get_by_key(&self, id: impl Into<Self::DataType> + Send) -> Result<Option<T>, Error> {
        let id = id.into();
        let mut builder = Select::columns(&["*"])
            .from(self.table_name)
            .where_(col(self.primary_key.0).eq(id));
        self.apply_global_filters(&mut builder);
        self.query.fetch_optional::<T>(builder).await
    }

    async fn get_one<F>(&self, query_condition: Option<F>) -> Result<Option<T>, Error>
    where
        F: FnOnce(&mut Self::QueryFilter<'a>) + Send + 'a,
    {
        let mut builder = Select::columns(&["*"]).from(self.table_name);
        self.apply_global_filters(&mut builder);
        if let Some(condition) = query_condition {
            condition(&mut builder);
        }
        self.query.fetch_optional::<T>(builder).await
    }

    async fn get_list_paginated<F>(
        &self,
        page_number: u64,
        page_size: u64,
        query_condition: Option<F>,
    ) -> Result<PaginatedResult<T>, Error> 
    where 
        F: FnOnce(&mut Self::QueryFilter<'a>) + Send + 'a
    {
        if page_number == 0 || page_size == 0 {
            return Err(OperationError::new("Page number and page size must be greater than 0".to_string()));
        }

        let offset = (page_number - 1) * page_size;
        let mut builder = Select::columns(&["*"])
            .from(self.table_name)
            .limit_offset(DataKind::from(page_size), Some(DataKind::from(offset)));
    
        self.apply_global_filters(&mut builder);
        let total = if let Some(condition) = query_condition {
            condition(&mut builder);
            let count_builder = builder.clone();
            self.count(Some(move |b: &mut Self::QueryFilter<'a>| 
                *b = count_builder
            )).await?
        } else {
            0
        };
    
        let data = self.query.fetch_all::<T>(builder).await?;
        Ok(PaginatedResult {
            data,
            total,
            page_number,
            page_size,
        })
    }

    async fn get_list_by_cursor<F>(
        &self,
        limit: u64,
        query_condition: Option<F>,
    ) -> Result<CursorPaginatedResult<T>, Error>
    where
        T: Clone,
        F: FnOnce(&mut Self::QueryFilter<'a>) + Send + 'a
    {
        if limit == 0 {
            return Err(OperationError::new("Limit must be greater than 0".to_string()));
        }
        let mut builder = Select::columns(&["*"])
            .from(self.table_name)
            .limit_offset(DataKind::from(limit), Some(DataKind::from(0)));

        self.apply_global_filters(&mut builder);
        if let Some(condition) = query_condition {
            condition(&mut builder);
        }
  
        let data = self.query.fetch_all::<T>(builder).await?;

        let next_cursor = data.last().cloned();
        Ok(CursorPaginatedResult {
            data,
            next_cursor,
            page_size: limit,
        })
    }

    async fn delete_by_key(&self, key: impl Into<Self::DataType> + Send) -> Result<Self::QueryResult, Error> {
        let key = key.into();
        let builder = Delete::from(self.table_name)
            .where_(col(self.primary_key.0).eq(key));
        self.query.execute(builder).await
    }

    async fn delete_many(&self, keys: Vec<impl Into<Self::DataType>>) -> Result<Self::QueryResult, Error>
    {
        if keys.is_empty() {
            return Err(OperationError::new("Keys list cannot be empty".to_string()));
        }
        if keys.len() > 1000 {
            return Err(OperationError::new("Keys list cannot exceed 1000 items".to_string()));
        }

        let mut builder = Delete::from(self.table_name)
            .where_(col(self.primary_key.0).in_(keys));
        self.apply_global_filters(&mut builder);
        self.query.execute(builder).await
    }

    async fn delete_by_cond<F>(&self, query_condition: Option<F>) -> Result<Self::QueryResult, Error>
    where
        F: FnOnce(&mut Self::DeleteFilter<'a>) + Send + 'a,
    {
        let mut builder = Delete::from(self.table_name);

        self.apply_global_filters(&mut builder);

        if let Some(condition) = query_condition {
            condition(&mut builder);
        }

        self.query.execute(builder).await
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

        if cols_names.is_empty() {
            return Err(OperationError::new("No valid fields provided for insertion".to_string()));
        }

        let builder = Insert::into(self.table_name)
            .columns(&cols_names)
            .values(vec![cols_values]);
        self.query.execute(builder).await
    }

    async fn insert_many(&self, entities: Vec<T>) -> Result<Self::QueryResult, Error> {
        if entities.is_empty() {
            return Err(OperationError::new("No entities provided for insert operation".to_string()));
        }

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

        let builder = Insert::into(self.table_name)
            .columns(&cols_names)
            .values(all_cols_values);
        self.query.execute(builder).await
    }

    async fn update_by_key(&self, entity: T) -> Result<Self::QueryResult, Error> {
        let mut cols_names = Vec::new();
        let mut cols_values = Vec::new();

        for (name, field) in entity.fields() {
            if name != self.primary_key.0 {
                cols_names.push(name);
                let value = value_convert(field.as_any());
                cols_values.push(value);
            }
        }

        if cols_names.is_empty() {
            return Err(OperationError::new("No updatable fields provided".to_string()));
        }

        let primary_key_value = entity.fields()
            .find(|(name, _)| *name == self.primary_key.0)
            .map(|(_, field)| value_convert(field.as_any()))
            .ok_or(OperationError::new(
                format!("Primary key {} not found", self.primary_key.0)
            ))?;

        let builder = Update::table(self.table_name)
            .set_cols(&cols_names, cols_values)
            .where_(col(self.primary_key.0).eq(primary_key_value));
        self.query.execute(builder).await
    }
    
    async fn update_one<F>(&self, entity: T, query_condition: Option<F>) -> Result<Self::QueryResult, Error>
    where
        F: FnOnce(&mut Self::UpdateFilter<'a>) + Send + 'a,
    {
        let mut cols_names = Vec::new();
        let mut cols_values = Vec::new();

        for (name, field) in entity.fields() {
            if name != self.primary_key.0 {
                let value = value_convert(field.as_any());
                cols_names.push(name);
                cols_values.push(value);
            }
        }

        let primary_key_value = entity.fields()
            .find(|(name, _)| *name == self.primary_key.0)
            .map(|(_, field)| value_convert(field.as_any()))
            .ok_or(Error::RowNotFound)?;

        let mut builder = Update::table(self.table_name)
            .set_cols(&cols_names, cols_values)
            .where_(col(self.primary_key.0).eq(primary_key_value));

        if let Some(condition) = query_condition {
            condition(&mut builder);
        }

        self.query.execute(builder).await
    }

    async fn upsert_one(&self, entity: T) -> Result<Self::QueryResult, Error> {
        let mut cols_names = Vec::new();
        let mut cols_values = Vec::new();

        for (name, field) in entity.fields() {
            if !cols_names.contains(&name) {
                cols_names.push(name);
            }

            let value = value_convert(field.as_any());
            cols_values.push(value);
        } 

        let conflict_target = self.primary_key.0;
        let builder = Insert::into(self.table_name)
            .columns(&cols_names)
            .values(vec![cols_values])
            .on_conflict_do_update(conflict_target, &cols_names);
            //.returning(&cols_names);

        self.query.execute(builder).await
    }

    async fn upsert_many(&self, entities: Vec<T>) -> Result<Self::QueryResult, Error> {
        if entities.is_empty() {
            return Err(OperationError::new("No entities provided for upsert operation".to_string()));
        }
    
        let mut cols_names = Vec::new();
        let mut all_cols_values = Vec::new();

        for (i, entity) in entities.iter().enumerate() {
            let mut cols_values = Vec::new();
    
            for (name, field) in entity.fields() {
                if i == 0 && !cols_names.contains(&name) {
                    cols_names.push(name);
                }

                let value = value_convert(field.as_any());
                cols_values.push(value);
            }            
    
            all_cols_values.push(cols_values);
        }

        let conflict_target = self.primary_key.0;
        let builder = Insert::into(self.table_name)
            .columns(&cols_names)
            .values(all_cols_values)
            .on_conflict_do_update(conflict_target, &cols_names);
            //.returning(&cols_names);

        self.query.execute(builder).await
    }

    async fn restore_one(&self, key: impl Into<Self::DataType> + Send) -> Result<Self::QueryResult, Error> {
        let key = key.into();
        if let Some((column, exclude_tables)) = get_global_soft_delete_field() {
            if !exclude_tables.contains(&self.table_name) {
                let query = Update::table(self.table_name)
                    .set_cols(&[column],vec![DataKind::from(false)])
                    .where_(col(self.primary_key.0).eq(key));
                return self.query.execute(query).await;
            }
        }
        Err(OperationError::new("Restore operation not supported without soft delete configuration".to_string()))
    }

    async fn restore_many(&self, keys: Vec<impl Into<Self::DataType> + Send>) -> Result<Self::QueryResult, Error> {
        let keys: Vec<DataKind<'a>> = keys.into_iter().map(|k| k.into()).collect();
        if let Some((column, exclude_tables)) = get_global_soft_delete_field() {
            if !exclude_tables.contains(&self.table_name) {
                let query = Update::table(self.table_name)
                    .set_cols(&[column], vec![DataKind::from(false)])
                    .where_(col(self.primary_key.0).in_(keys));
                return self.query.execute(query).await;
            }
        }
        Err(OperationError::new("Restore operation not supported without soft delete configuration".to_string()))
    }

    async fn exist<F>(&self, query_condition: Option<F>) -> Result<bool, Error> 
    where 
        F: FnOnce(&mut Self::QueryFilter<'a>) + Send + 'a
    {
        let mut builder = Select::columns(&["1"]).from(self.table_name);
        self.apply_global_filters(&mut builder);
        if let Some(condition) = query_condition {
            condition(&mut builder);
        }        
        let result = self.query.fetch_optional::<(i32,)>(builder).await?;
        Ok(result.is_some())
    }

    async fn count<F>(&self, query_condition: Option<F>) -> Result<i64, Error> 
    where 
        F: FnOnce(&mut Self::QueryFilter<'a>) + Send + 'a
    {
        let mut builder = Select::columns(&["COUNT(*)"]).from(self.table_name);
        if let Some(condition) = query_condition {
            condition(&mut builder);
        }
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
    fn apply_global_filters<W>(&self, builder: &mut W)
    where
        W: FilterTrait<DataKind<'a>, Expr = Expr<DataKind<'a>>> + 'a,
    {

        if let Some((soft_delete_field, exclude_tables)) = get_global_soft_delete_field() {
            if !exclude_tables.contains(&self.table_name) {
                builder.where_mut(col(soft_delete_field).eq(false));
            }
        }

        if let Some((filter, exclude_tables)) = get_global_filter() {
            if !exclude_tables.contains(&self.table_name) {
                builder.where_mut(filter);
            }
        }
    }
}