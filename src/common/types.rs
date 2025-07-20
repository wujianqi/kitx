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

#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
#[derive(Debug, Clone)]
pub enum PrimaryKey<D> {
    SingleKey(D),
    CompositeKey(Vec<D>),
}

#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
impl<D> From<D> for PrimaryKey<D>
where
    D: Clone + Debug + Send,
{
    fn from(value: D) -> Self {
        PrimaryKey::SingleKey(value)
    }
}

#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
impl<D> From<Vec<D>> for PrimaryKey<D>
where
    D: Clone + Debug + Send,
{
    fn from(values: Vec<D>) -> Self {
        PrimaryKey::CompositeKey(values)
    }
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
pub struct CursorPaginatedResult<T, C> {
    pub data: Vec<T>,
    pub next_cursor: Option<C>,
    pub limit: u64,
}