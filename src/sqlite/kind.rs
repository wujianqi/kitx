use std::any::Any;
use std::borrow::Cow;
use std::error::Error;
use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::encode::IsNull;
use sqlx::{Database, Encode, Sqlite, Type};
use sqlx::sqlite::SqliteArgumentValue;

use crate::common::util::{check_empty, unwrap_option};

/// Enum representing different types of database field values.
#[derive(Debug, Clone)]
pub enum DataKind<'a> {
    /// Text type (string).
    Text(Cow<'a, str>),
    /// Integer type (i64).
    Integer(i64),
    /// Real number type (f64).
    Real(f64),
    /// Date and time type (`DateTime<Utc>`).
    DateTime(DateTime<Utc>),
    /// BLOB type (byte array).
    Blob(Cow<'a, [u8]>),
    /// Null type.
    Null,
    /// Unsupported type.
    Unsupported,
}

impl<'a> Encode<'a, Sqlite> for DataKind<'a> {
    fn encode_by_ref(&self, buf: &mut Vec<SqliteArgumentValue<'_>>) -> Result<IsNull, Box<dyn Error + Send + Sync + 'static>> {
        match self {
            DataKind::Text(text) => {
                buf.push(SqliteArgumentValue::Text(text.to_string().into()));
                Ok(IsNull::No)
            },
            DataKind::Integer(int) => {
                buf.push(SqliteArgumentValue::Int64(*int));
                Ok(IsNull::No)
            },
            DataKind::Real(real) => {
                buf.push(SqliteArgumentValue::Double(*real));
                Ok(IsNull::No)
            },
            DataKind::DateTime(datetime) => {
                let rfc3339 = datetime.to_rfc3339();
                buf.push(SqliteArgumentValue::Text(rfc3339.into()));
                Ok(IsNull::No)
            },
            DataKind::Blob(blob) => {
                buf.push(SqliteArgumentValue::Blob(blob.to_vec().into()));
                Ok(IsNull::No)
            },
            DataKind::Null => {
                buf.push(SqliteArgumentValue::Null);
                Ok(IsNull::Yes)
            },
            DataKind::Unsupported => Err("Unsupported data kind cannot be encoded".into()),
        }
    }
}

impl<'a> Type<Sqlite> for DataKind<'a> {
    fn type_info() -> <Sqlite as Database>::TypeInfo {
        <str as Type<Sqlite>>::type_info()
    }

    fn compatible(_: &<Sqlite as Database>::TypeInfo) -> bool {
        true
    }
}

/// Convert any type of value to the `DataKind` enum type.
pub fn value_convert<'a>(value: &dyn Any) -> DataKind<'a> {
    if let Some(s) = unwrap_option::<String>(value) {
        DataKind::Text(Cow::Owned(s.clone()))
    } else if let Some(s) = unwrap_option::<&str>(value) {
        DataKind::Text(Cow::Borrowed(s))
    } else if let Some(i) = unwrap_option::<i64>(value) {
        DataKind::Integer(*i)
    } else if let Some(u) = unwrap_option::<u64>(value) {
        DataKind::Integer(*u as i64)
    } else if let Some(i) = unwrap_option::<i32>(value) {
        DataKind::Integer(*i as i64)
    } else if let Some(u) = unwrap_option::<u32>(value) {
        DataKind::Integer(*u as i64)
    } else if let Some(b) = unwrap_option::<bool>(value) {
        DataKind::Integer(*b as i64)
    } else if let Some(r) = unwrap_option::<f64>(value) {
        DataKind::Real(*r)
    } else if let Some(r) = unwrap_option::<f32>(value) {
        DataKind::Real(*r as f64)
    } else if let Some(dt) = unwrap_option::<DateTime<Utc>>(value) {
        DataKind::DateTime(*dt)
    } else if let Some(ndt) = unwrap_option::<NaiveDateTime>(value) {
        DataKind::DateTime(DateTime::from_naive_utc_and_offset(*ndt, Utc))
    } else if let Some(blob) = unwrap_option::<Vec<u8>>(value) {
        DataKind::Blob(Cow::Owned(blob.clone()))
    } else if let Some(blob) = unwrap_option::<&[u8]>(value) {
        DataKind::Blob(Cow::Borrowed(blob))
    } else if value.is::<Option<String>>()
           || value.is::<Option<&str>>()
           || value.is::<Option<i64>>()
           || value.is::<Option<u64>>()
           || value.is::<Option<i32>>()
           || value.is::<Option<u32>>()
           || value.is::<Option<bool>>()
           || value.is::<Option<f64>>()
           || value.is::<Option<f32>>()
           || value.is::<Option<DateTime<Utc>>>()
           || value.is::<Option<NaiveDateTime>>()
           || value.is::<Option<&[u8]>>()
           || value.is::<Option<Vec<u8>>>() {
        DataKind::Null
    } else {
        DataKind::Unsupported
    }
}

/// Helper function to determine if a value is empty.
pub fn is_empty(value: &dyn Any) -> bool {
    check_empty(value, |value| {
        if value.is::<Option<String>>()
        || value.is::<Option<&str>>()
        || value.is::<Option<i32>>()
        || value.is::<Option<u32>>()
        || value.is::<Option<bool>>()
        || value.is::<Option<f32>>()
        || value.is::<Option<i64>>()
        || value.is::<Option<f64>>()
        || value.is::<Option<DateTime<Utc>>>()
        || value.is::<Option<NaiveDateTime>>()
        || value.is::<Option<&[u8]>>()
        || value.is::<Option<Vec<u8>>>() {
            return unwrap_option::<()>(value).is_none();
        }
        false
    })
}

// Implement automatic conversion from common types to DataKind
impl<'a> From<Cow<'a, str>> for DataKind<'a> {
    fn from(item: Cow<'a, str>) -> Self {
        DataKind::Text(item)
    }
}

impl From<String> for DataKind<'_> {
    fn from(item: String) -> Self {
        DataKind::Text(Cow::Owned(item))
    }
}

impl<'a> From<&'a str> for DataKind<'a> {
    fn from(item: &'a str) -> Self {
        DataKind::Text(Cow::Borrowed(item))
    }
}

impl From<i32> for DataKind<'_> {
    fn from(item: i32) -> Self {
        DataKind::Integer(item as i64)
    }
}

impl From<i64> for DataKind<'_> {
    fn from(item: i64) -> Self {
        DataKind::Integer(item)
    }
}

impl From<bool> for DataKind<'_> {
    fn from(item: bool) -> Self {
        DataKind::Integer(item as i64)
    }
}

impl From<f32> for DataKind<'_> {
    fn from(item: f32) -> Self {
        DataKind::Real(item as f64)
    }
}

impl From<f64> for DataKind<'_> {
    fn from(item: f64) -> Self {
        DataKind::Real(item)
    }
}

impl From<DateTime<Utc>> for DataKind<'_> {
    fn from(item: DateTime<Utc>) -> Self {
        DataKind::DateTime(item)
    }
}

impl From<NaiveDateTime> for DataKind<'_> {
    fn from(item: NaiveDateTime) -> Self {
        DataKind::DateTime(DateTime::from_naive_utc_and_offset(item, Utc))
    }
}

impl<'a> From<Cow<'a, [u8]>> for DataKind<'a> {
    fn from(item: Cow<'a, [u8]>) -> Self {
        DataKind::Blob(item)
    }
}

impl From<Vec<u8>> for DataKind<'_> {
    fn from(item: Vec<u8>) -> Self {
        DataKind::Blob(Cow::Owned(item))
    }
}

impl<'a> From<&'a [u8]> for DataKind<'a> {
    fn from(item: &'a [u8]) -> Self {
        DataKind::Blob(Cow::Borrowed(item))
    }
}

impl From<u32> for DataKind<'_> {
    fn from(item: u32) -> Self {
        DataKind::Integer(item as i64)
    }
}

impl From<u64> for DataKind<'_> {
    fn from(item: u64) -> Self {
        DataKind::Integer(item as i64)
    }
}