use std::future::Future;
use serde::{Deserialize, Serialize};
use sqlx::{Database, Error, FromRow};

pub trait OperationsTrait<'a, T, DB>: Send + Sync 
where
    DB: Database,
    T: for<'r> FromRow<'r, DB::Row> + Send + Sync + Default,
{
    type Query;
    type DataKind;
    type QueryResult;

    /// Creates a new `Operations` instance.
    /// 
    /// # Parameters
    /// * `table_name`: The name of the table.
    /// * `primary_key`: A tuple containing the name of the primary key column and a boolean indicating whether the primary key is auto-incrementing.
    /// * `soft_delete_info`: A tuple containing the name of the soft-delete column and a boolean indicating whether the column is auto-incrementing.
    fn new(table_name: &'a str, primary_key: (&'a str, bool), soft_delete_info: Option<(&'a str, bool)>) -> Self;

    /// Inserts a single record into the database and returns the primary key value of the inserted record.
    /// 
    /// # Parameters
    /// * `entity`: The entity to be inserted.
    /// 
    /// # Returns
    /// Returns the primary key value of the inserted record.
    fn insert_one(&self, entity: T) -> impl Future<Output = Result<Self::QueryResult, Error>> + Send;

    /// Inserts multiple records into the database and returns a list of primary key values of the inserted records.
    /// 
    /// # Parameters
    /// * `entities`: A list of entities to be inserted.
    /// 
    /// # Returns
    /// Returns a list of primary key values of the inserted records.
    fn insert_many(&self, entities: Vec<T>) -> impl Future<Output = Result<Self::QueryResult, Error>> + Send;

    /// Updates a single record and returns the number of affected rows.
    /// 
    /// # Parameters
    /// * `entity`: The entity to be updated.
    /// * `override_empty`: A boolean indicating whether to update fields with empty values.
    /// 
    /// # Returns
    /// Returns the number of affected rows.
    fn update_one(&self, entity: T, override_empty: bool) -> impl Future<Output = Result<Self::QueryResult, Error>> + Send;

    /// Updates multiple records and returns a list of the number of affected rows.
    /// 
    /// # Parameters
    /// * `entities`: A list of entities to be updated.
    /// * `override_empty`: A boolean indicating whether to update fields with empty values.
    /// 
    /// # Returns
    /// Returns a list of the number of affected rows.
    fn update_many(&self, entities: Vec<T>, override_empty: bool) -> impl Future<Output = Result<Vec<Self::QueryResult>, Error>> + Send;

    /// Deletes a single record and returns the number of affected rows.
    /// 
    /// # Parameters
    /// * `key`: The primary key value of the record to be deleted.
    /// 
    /// # Returns
    /// Returns the number of affected rows.
    fn delete_one(&self, key: impl Into<Self::DataKind> + Send) -> impl Future<Output = Result<Self::QueryResult, Error>> + Send;

    /// Deletes multiple records and returns the number of affected rows.
    /// 
    /// # Parameters
    /// * `keys`: A list of primary key values of the records to be deleted.
    /// 
    /// # Returns
    /// Returns the number of affected rows.
    fn delete_many(&self, keys: Vec<impl Into<Self::DataKind> + Send>) -> impl Future<Output = Result<Self::QueryResult, Error>> + Send;

    /// Queries and returns all records in the table, supporting conditional queries.
    /// 
    /// # Parameters
    /// * `query_condition`: A query condition structure.
    /// 
    /// # Returns
    /// Returns a list of records.
    fn fetch_all(&self, query_condition: Self::Query) -> impl Future<Output = Result<Vec<T>, Error>> + Send;

    /// Queries and returns a single record based on the primary key.
    /// 
    /// # Parameters
    /// * `id`: The primary key value of the record to be queried.
    /// 
    /// # Returns
    /// Returns a single record.
    fn fetch_by_key(&self, id: impl Into<Self::DataKind> + Send) -> impl Future<Output = Result<Option<T>, Error>> + Send;

    /// Queries and returns a single record based on field conditions.
    /// 
    /// # Parameters
    /// * `query_condition`: A query condition structure.
    /// 
    /// # Returns
    /// Returns a single record.
    fn fetch_one(&self, query_condition: Self::Query) -> impl Future<Output = Result<Option<T>, Error>> + Send;

    /// Paginates and returns records in the table, supporting conditional queries.
    /// 
    /// # Parameters
    /// * `page_number`: The page number.
    /// * `page_size`: The number of records per page.
    /// * `query_condition`: A query condition structure.
    /// 
    /// # Returns
    /// Returns a paginated result structure.
    fn fetch_paginated(&self, page_number: u64, page_size: u64, query_condition: Self::Query) -> impl Future<Output = Result<PaginatedResult<T>, Error>> + Send;

    /// Cursor paginates and returns records in the table, supporting conditional queries.
    /// 
    /// # Parameters
    /// * `limit`: The number of records per page.
    /// * `query_condition`: A query condition structure.
    /// 
    /// # Returns
    /// Returns a cursor paginated result structure.
    fn fetch_by_cursor(&self, limit: u64, query_condition: Self::Query) -> impl Future<Output = Result<CursorPaginatedResult<T>, Error>> + Send  where T: Clone;

    /// Checks if the value of a field is unique.
    /// 
    /// # Parameters
    /// * `query_condition`: A query condition structure.
    fn exist(&self, query_condition: Self::Query) -> impl Future<Output = Result<bool, Error>> + Send;

    /// Gets the total number of records, supporting conditional queries.
    /// 
    /// # Parameters
    /// * `query_condition`: A query condition structure.
    /// 
    /// # Returns
    /// Returns the total number of records.
    fn count(&self, query_condition: Self::Query) -> impl Future<Output = Result<i64, Error>> + Send;

    /// Restores a single soft-deleted record.
    /// 
    /// # Parameters
    /// * `key`: The primary key value of the record to be restored.
    fn restore_one(&self, key: impl Into<Self::DataKind> + Send) -> impl Future<Output = Result<Self::QueryResult, Error>> + Send;

    /// Restores multiple soft-deleted records.
    /// 
    /// # Parameters
    /// * `keys`: A list of primary key values of the records to be restored.
    fn restore_many(&self, keys: Vec<impl Into<Self::DataKind> + Send>) -> impl Future<Output = Result<Self::QueryResult, Error>> + Send;

}

/// Paginated query result structure.
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Hash)]
pub struct PaginatedResult<T> {
    /// Data records queried.
    pub data: Vec<T>,
    /// Total number of records.
    pub total: i64,
    pub page_number: u64,
    pub page_size: u64,
}

/// Cursor paginated result structure.
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Hash)]
pub struct CursorPaginatedResult<T> {
    pub data: Vec<T>,      // Paginated data.
    pub next_cursor: Option<T>, // Next cursor value.
    pub page_size: u64,
}