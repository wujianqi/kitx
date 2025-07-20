use std::fmt::Debug;
use std::future::Future;

use sqlx::{Database, Error, FromRow};
use super::types::{PrimaryKey, CursorPaginatedResult, PaginatedResult};

/// Trait for building operations on entities
/// This trait defines methods for inserting, updating, deleting, and querying entities.
/// It is generic over the entity type `T` and the database type `D`.
/// The trait is designed to be used with a specific database type that implements the `Database` trait from `sqlx`.
pub trait OpsBuilderTrait<'a, T, D> 
where 
    T: Send + Sync + 'a,
    D: Clone + Debug + Send +  'a,
{
    type SelectBuilder;
    type UpdateBuilder;
    type DeleteBuilder;
    type InsertBuilder;

    fn insert_many(&self, entities: Vec<T>) -> Result<Self::InsertBuilder, Error>;
    fn update_one(&self, entity: T) -> Result<Self::UpdateBuilder, Error>;
    fn update_by_cond<F>(&self, query_condition: F) -> Result<Self::UpdateBuilder, Error>
        where F: Fn(&mut Self::UpdateBuilder) + Send;
    fn upsert_many(&self, entities: Vec<T>, use_default_expr: bool) -> Result<(Self::InsertBuilder, Vec<&'a str>, Vec<&'a str>), Error>;

    fn delete_by_pk(&self, key: impl Into<PrimaryKey<D>>) -> Result<Self::DeleteBuilder, Error>;
    fn delete_by_cond<F>(&self, query_condition: F) -> Result<Self::DeleteBuilder, Error>
        where F: Fn(&mut Self::DeleteBuilder) + Send;

    fn fetch_by_cond<F>(&self, query_condition: F) -> Self::SelectBuilder
        where F: Fn(&mut Self::SelectBuilder);
    fn fetch_by_pk(&self, key: impl Into<PrimaryKey<D>>) -> Result<Self::SelectBuilder, Error>;    
    fn get_list_paginated<F>(&self, page_number: u64, page_size: u64, query_condition: F) -> Result<Self::SelectBuilder, Error>
        where F: Fn(&mut Self::SelectBuilder);
    fn get_list_by_cursor<F>(&self, limit: u64, query_condition: F) -> Result<Self::SelectBuilder, Error>
        where F: Fn(&mut Self::SelectBuilder);
    fn exists<F>(&self, query_condition: F) -> Self::SelectBuilder
        where F: Fn(&mut Self::SelectBuilder);
    fn count<F>(&self, query_condition: F) -> Self::SelectBuilder
        where F: Fn(&mut Self::SelectBuilder);    

    // Soft delete and restore operations
    fn soft_delete_by_pk(&self, key: impl Into<PrimaryKey<D>>) -> Result<Self::UpdateBuilder, Error>;
    fn soft_delete_by_cond<F>(&self, query_condition: F) -> Result<Self::UpdateBuilder, Error>
        where F: Fn(&mut Self::UpdateBuilder) + Send;
    fn restore_by_pk(&self, key: impl Into<PrimaryKey<D>>) -> Result<Self::UpdateBuilder, Error>;
    fn restore_by_cond<F>(&self, query_condition: F) -> Result<Self::UpdateBuilder, Error>
       where F: Fn(&mut Self::UpdateBuilder) + Send;

    // Soft delete status check
    fn is_soft_delete_enabled(&self) -> bool;
}

/// Trait for performing operations on entities
/// This trait defines methods for inserting, updating, deleting, and querying entities.
/// It is generic over the entity type `T`, the database type `DB`, and the primary key type `D`.
/// It is designed to be used with a specific database type that implements the `Database` trait from `sqlx`.
pub trait OpsActionTrait<'a, T, DB, D>: Send + Sync 
where
    DB: Database,
    T: Default + for<'r> FromRow<'r, DB::Row> + Send + Sync,
    D: Clone + Debug + Send + Sync,
{
    type QueryFilter<'b>;
    type UpdateFilter<'b>;
    type DeleteFilter<'b>;

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
    fn update_one(&self, entity: T) -> impl Future<Output = Result<DB::QueryResult, Error>> + Send;

    /// Updates a single record and returns the number of affected rows.
    /// 
    /// # Parameters
    /// * `query_condition`: The query condition to filter the records to be updated.
    /// 
    /// # Returns
    /// Returns the number of affected rows.
    fn update_by_cond<F>(&self, query_condition: F) -> impl Future<Output = Result<DB::QueryResult, Error>> + Send
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
    fn delete_by_pk(&self, key: impl Into<PrimaryKey<D>> + Send + Sync) -> impl Future<Output = Result<DB::QueryResult, Error>> + Send;

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

    /// Queries and returns a single record based on the primary key.
    /// 
    /// # Parameters
    /// * `key`: The primary key value of the record to be queried.
    /// 
    /// # Returns
    /// Returns a single record.
    fn get_one_by_pk(&self, key: impl Into<PrimaryKey<D>> + Send + Sync) -> impl Future<Output = Result<Option<T>, Error>> + Send;

    /// Queries and returns a single record based on field conditions.
    /// 
    /// # Parameters
    /// * `query_condition`: A query condition structure.
    /// 
    /// # Returns
    /// Returns a single record.
    fn get_one_by_cond<F>(&self, query_condition: F) -> impl Future<Output = Result<Option<T>, Error>> + Send
    where
        F: Fn(&mut Self::QueryFilter<'a>) + Send + Sync + 'a;

    /// Queries and returns all records in the table, supporting conditional queries.
    /// 
    /// # Parameters
    /// * `query_condition`: A query condition structure.
    /// 
    /// # Returns
    /// Returns a list of records.
    fn get_list_by_cond<F>(&self, query_condition: F) -> impl Future<Output = Result<Vec<T>, Error>> + Send
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
    fn get_list_by_cursor<F, C>(
        &self,
        limit: u64,
        query_condition: F,
        cursor_extractor: impl Fn(&T) -> C + Send + Sync,
    ) -> impl Future<Output = Result<CursorPaginatedResult<T, C>, Error>> + Send
    where
        F: Fn(&mut Self::QueryFilter<'a>) + Send + Sync + 'a,
        C: Send + Sync;

    /// Checks if the value of a field is unique.
    /// 
    /// # Parameters
    /// * `query_condition`: A query condition structure.
    fn exists<F>(&self, query_condition: F) -> impl Future<Output = Result<bool, Error>> + Send
    where
        F: Fn(&mut Self::QueryFilter<'a>) + Send + Sync + 'a;

    /// Gets the total number of records, supporting conditional queries.
    /// 
    /// # Parameters
    /// * `query_condition`: A query condition structure.
    /// 
    /// # Returns
    /// Returns the total number of records.
    fn count<F>(&self, query_condition: F) -> impl Future<Output = Result<u64, Error>> + Send
    where
        F: Fn(&mut Self::QueryFilter<'a>) + Send + Sync + 'a;

    /// Restores a single soft-deleted record.
    /// 
    /// # Parameters
    /// * `key`: The primary key value of the record to be restored.
    fn restore_by_pk(&self, key: impl Into<PrimaryKey<D>> + Send + Sync) -> impl Future<Output = Result<DB::QueryResult, Error>> + Send;

    /// Restores soft-deleted records based on a query condition.
    fn restore_by_cond<F>(&self, query_condition: F) -> impl Future<Output = Result<DB::QueryResult, Error>> + Send
     where
        F: Fn(&mut Self::UpdateFilter<'a>) + Send + Sync + 'a;

}
