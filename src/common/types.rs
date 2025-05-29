use std::fmt::Debug;

#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
use serde::{Deserialize, Serialize};

/// Order by direction.
#[derive(Default, Debug, Clone)]
pub enum OrderBy {
    #[default]
    Asc,
    Desc,
}

/// Paginated query result structure.
#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
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
#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Hash)]
pub struct CursorPaginatedResult<T> {
    pub data: Vec<T>,      // Paginated data.
    pub next_cursor: Option<T>, // Next cursor value.
    pub page_size: u64,
}
