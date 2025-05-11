use std::fmt::Debug;
use serde::{Deserialize, Serialize};

/// Order by direction.
#[derive(Default, Debug, Clone)]
pub enum OrderBy {
    #[default]
    Asc,
    Desc,
}

///Primary key type.
#[derive(Debug, Clone)]
pub enum PrimaryKey<'a> {
    Single(&'a str, bool),
    Composite(Vec<&'a str>)
}

/// Paginated query result structure.
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Hash)]
pub struct PaginatedResult<T> {
    /// Data records queried.
    pub data: Vec<T>,
    /// Total number of records.
    pub total: u64,
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
