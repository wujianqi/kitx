use std::fmt::Debug;
use std::future::Future;

use serde::{Deserialize, Serialize};
use sqlx::{Database, Error, FromRow};

pub trait OperationsTrait<'a, T, DB, D>: Send + Sync 
where
    DB: Database,
    T: for<'r> FromRow<'r, DB::Row> + Send + Sync + Default,
    D: Clone + Debug + Send + Sync,
{
    type QueryFilter<'b>;
    type UpdateFilter<'b>;
    type DeleteFilter<'b>;

    /// Creates a new `Operations` instance.
    fn new(table_name: &'a str, primary_key: (&'a str, bool)) -> Self;

    /// Inserts a single record into the database and returns the primary key value of the inserted record.
    /// 
    /// # Parameters
    /// * `entity`: The entity to be inserted.
    /// 
    /// # Returns
    /// Returns the primary key value of the inserted record.
    fn insert_one(&self, entity: T) -> impl Future<Output = Result<DB::QueryResult, Error>> + Send;

    /// Inserts multiple records into the database and returns a list of primary key values of the inserted records.
    /// 
    /// # Parameters
    /// * `entities`: A list of entities to be inserted.
    /// 
    /// # Returns
    /// Returns a list of primary key values of the inserted records.
    fn insert_many(&self, entities: Vec<T>) -> impl Future<Output = Result<DB::QueryResult, Error>> + Send;

    /// Updates a single record and returns the number of affected rows.
    /// 
    /// # Parameters
    /// * `entity`: The entity to be updated.
    /// 
    /// # Returns
    /// Returns the number of affected rows.
    fn update_by_key(&self, entity: T) -> impl Future<Output = Result<DB::QueryResult, Error>> + Send;

    /// Updates a single record and returns the number of affected rows.
    /// 
    /// # Parameters
    /// * `entity`: The entity to be updated.
    /// * `query_condition`: The query condition to filter the records to be updated.
    /// 
    /// # Returns
    /// Returns the number of affected rows.
    fn update_one<F>(&self, entity: T, query_condition: F) -> impl Future<Output = Result<DB::QueryResult, Error>> + Send
    where
        F: Fn(&mut Self::UpdateFilter<'a>) + Send + Sync + 'a;

    /// Upserts a record into the database.
    /// If the record already exists, it updates the record.
    /// If the record does not exist, it inserts the record.
    /// # Parameters
    /// - `entity`: The record to be upserted.
    fn upsert_one(&self, entity: T) -> impl Future<Output = Result<DB::QueryResult, Error>> + Send;

    /// Upserts multiple records into the database.
    /// If the records already exist, it updates the records.
    /// If the records do not exist, it inserts the records.
    /// # Parameters
    /// - `entities`: The records to be upserted.
    fn upsert_many(&self, entities: Vec<T>) -> impl Future<Output = Result<DB::QueryResult, Error>> + Send;

    /// Deletes a single record and returns the number of affected rows.
    /// 
    /// # Parameters
    /// * `key`: The primary key value of the record to be deleted.
    /// 
    /// # Returns
    /// Returns the number of affected rows.
    fn delete_by_key(&self, key: impl Into<D> + Send) -> impl Future<Output = Result<DB::QueryResult, Error>> + Send;

    /// Deletes multiple records and returns the number of affected rows.
    /// 
    /// # Parameters
    /// * `keys`: A list of primary key values of the records to be deleted.
    /// 
    /// # Returns
    /// Returns the number of affected rows.
    fn delete_many(&self, keys: Vec<impl Into<D> + Send>) -> impl Future<Output = Result<DB::QueryResult, Error>> + Send;

    /// Deletes a single or multiple records and returns the number of affected rows.
    /// 
    /// # Parameters
    /// * `query_condition`: A query condition structure.
    /// 
    /// # Returns
    /// Returns the number of affected rows.
    ///
    fn delete_by_cond<F>(&self, query_condition: F) -> impl Future<Output = Result<DB::QueryResult, Error>> + Send
    where
        F: Fn(&mut Self::DeleteFilter<'a>) + Send + Sync + 'a;


    /// Queries and returns all records in the table, supporting conditional queries.
    /// 
    /// # Parameters
    /// * `query_condition`: A query condition structure.
    /// 
    /// # Returns
    /// Returns a list of records.
    fn get_list<F>(&self, query_condition: F) -> impl Future<Output = Result<Vec<T>, Error>> + Send
    where
        F: Fn(&mut Self::QueryFilter<'a>) + Send + Sync + 'a;

    /// Queries and returns a single record based on the primary key.
    /// 
    /// # Parameters
    /// * `id`: The primary key value of the record to be queried.
    /// 
    /// # Returns
    /// Returns a single record.
    fn get_by_key(&self, id: impl Into<D> + Send) -> impl Future<Output = Result<Option<T>, Error>> + Send;

    /// Queries and returns a single record based on field conditions.
    /// 
    /// # Parameters
    /// * `query_condition`: A query condition structure.
    /// 
    /// # Returns
    /// Returns a single record.
    fn get_one<F>(&self, query_condition: F) -> impl Future<Output = Result<Option<T>, Error>> + Send
    where
        F: Fn(&mut Self::QueryFilter<'a>) + Send + Sync + 'a;
    /// Paginates and returns records in the table, supporting conditional queries.
    /// 
    /// # Parameters
    /// * `page_number`: The page number.
    /// * `page_size`: The number of records per page.
    /// * `query_condition`: A query condition structure.
    /// 
    /// # Returns
    /// Returns a paginated result structure.
    fn get_list_paginated<F>(
        &self,
        page_number: u64,
        page_size: u64,
        query_condition: F,
    ) -> impl Future<Output = Result<PaginatedResult<T>, Error>> + Send
    where
        F: Fn(&mut Self::QueryFilter<'a>) + Send + Sync + 'a;

    /// Cursor paginates and returns records in the table, supporting conditional queries.
    /// 
    /// # Parameters
    /// * `limit`: The number of records per page.
    /// * `query_condition`: A query condition structure.
    /// 
    /// # Returns
    /// Returns a cursor paginated result structure.
    fn get_list_by_cursor<F>(
        &self,
        limit: u64,
        query_condition: F,
    ) -> impl Future<Output = Result<CursorPaginatedResult<T>, Error>> + Send
    where
        T: Clone,
        F: Fn(&mut Self::QueryFilter<'a>) + Send + Sync + 'a;

    /// Checks if the value of a field is unique.
    /// 
    /// # Parameters
    /// * `query_condition`: A query condition structure.
    fn exist<F>(&self, query_condition: F) -> impl Future<Output = Result<bool, Error>> + Send
    where
        F: Fn(&mut Self::QueryFilter<'a>) + Send + Sync + 'a;

    /// Gets the total number of records, supporting conditional queries.
    /// 
    /// # Parameters
    /// * `query_condition`: A query condition structure.
    /// 
    /// # Returns
    /// Returns the total number of records.
    fn count<F>(&self, query_condition: F) -> impl Future<Output = Result<i64, Error>> + Send
    where
        F: Fn(&mut Self::QueryFilter<'a>) + Send + Sync + 'a;

    /// Restores a single soft-deleted record.
    /// 
    /// # Parameters
    /// * `key`: The primary key value of the record to be restored.
    fn restore_one(&self, key: impl Into<D> + Send) -> impl Future<Output = Result<DB::QueryResult, Error>> + Send;

    /// Restores multiple soft-deleted records.
    /// 
    /// # Parameters
    /// * `keys`: A list of primary key values of the records to be restored.
    fn restore_many(&self, keys: Vec<impl Into<D> + Send>) -> impl Future<Output = Result<DB::QueryResult, Error>> + Send;

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
