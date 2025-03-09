#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
use field_access::FieldAccess;
#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
use serde::{Deserialize, Serialize};
#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
use sqlx::FromRow;
//use chrono::{DateTime, Utc};

#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
#[derive(Debug, Serialize, Deserialize, Default, FromRow, FieldAccess, Clone, PartialEq, Hash)]
pub struct Article {
    pub a_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub a_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub a_content: Option<String>,
}

#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
impl Article {
    #[allow(dead_code)]
    pub fn new(a_class: &str, a_content: &str, a_id: Option<i32>) -> Self {
        Article {
            a_class: Some(a_class.to_string()),
            a_content: Some(a_content.to_string()),
            a_id: a_id.unwrap_or(0),
        }
    }
}