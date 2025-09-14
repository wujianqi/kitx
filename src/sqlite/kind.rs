//! Data type definitions and conversions for SQLite database operations.
//! 
//! This module provides the [DataKind] enumeration which represents various database field types
//! supported by SQLite, along with their encoding and type conversion implementations. It handles
//! the mapping between Rust types and SQLite data types, including text, integer, real, blob,
//! date/time, boolean, JSON, and UUID types.
//! 
//! SQLite 数据库操作的数据类型定义和转换。
//! 
//! 本模块提供了 [DataKind] 枚举，用于表示 SQLite 支持的各种数据库字段类型，
//! 并包含它们的编码和类型转换实现。它处理 Rust 类型和 SQLite 数据类型之间的映射，
//! 包括文本、整数、实数、二进制数据、日期/时间、布尔值、JSON 和 UUID 类型。

use std::any::Any;
use std::borrow::Cow;
use std::error::Error;
use std::sync::Arc;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use serde_json::Value;
use sqlx::encode::IsNull;
use sqlx::types::Uuid;
use sqlx::{Database, Encode, Sqlite, Type};
use sqlx::sqlite::SqliteArgumentValue;

use crate::common::conversion::{unwrap_option, ValueConvert};

/// Enum representing different types of database field values.
#[derive(Default, Debug, Clone, PartialEq)]
pub enum DataKind {
    /// Text type (string).
    Text(String), // SQLite: TEXT

    /// Integer type.
    Integer(i64), // SQLite: INTEGER (includes i8, i16, i32, u8, u16, u32)

    /// Real number type.
    Real(f64), // SQLite: REAL

    /// Date and time types
    DateTime(NaiveDateTime), // SQLite: DATETIME (TEXT, INTEGER, REAL)
    DateTimeUtc(DateTime<Utc>), // SQLite: DATETIME (TEXT, INTEGER, REAL)
    Date(NaiveDate), // SQLite: DATE (TEXT only)
    Time(NaiveTime), // SQLite: TIME (TEXT only)

    /// BLOB type (byte array) - stored as Arc<[u8]> for zero-copy cloning
    Blob(Arc<[u8]>), // SQLite: BLOB

    /// Boolean type.
    Bool(bool), // SQLite: BOOLEAN (internally stored as INTEGER)

    /// JSON type (unstructured JSON data) - stored as `Arc<Value>` for zero-copy cloning
    Json(Arc<Value>), // SQLite: TEXT (JSON stored as text)

    /// UUID type (stored as BLOB or TEXT).
    Uuid(Uuid), // SQLite: BLOB or TEXT

    /// Null type.
    #[default]
    Null, // SQLite: NULL
}

impl Encode<'_, Sqlite> for DataKind {
    fn encode_by_ref(&self, buf: &mut Vec<SqliteArgumentValue<'_>>) -> Result<IsNull, Box<dyn Error + Send + Sync + 'static>> {
        match self {
            // Basic types
            DataKind::Null => Ok(IsNull::Yes),
            DataKind::Text(s) => <String as Encode<'_, Sqlite>>::encode(s.to_string(), buf),
            DataKind::Integer(i) => <i64 as Encode<'_, Sqlite>>::encode(*i, buf),
            DataKind::Real(r) => <f64 as Encode<'_, Sqlite>>::encode(*r, buf),

            // Date and time types
            DataKind::DateTime(dt) => {
                let utc_datetime = Utc.from_utc_datetime(dt);
                <String as Encode<'_, Sqlite>>::encode(utc_datetime.to_rfc3339(), buf)
            },
            DataKind::DateTimeUtc(dt_utc) => <String as Encode<'_, Sqlite>>::encode(dt_utc.to_rfc3339(), buf),
            DataKind::Date(date) => <String as Encode<'_, Sqlite>>::encode(date.format("%Y-%m-%d").to_string(), buf),
            DataKind::Time(time) => <String as Encode<'_, Sqlite>>::encode(time.format("%H:%M:%S%.f").to_string(), buf),

            DataKind::Blob(blob) => {
                let owned_blob = Vec::from(blob.as_ref());
                <Vec<u8> as Encode<'_, Sqlite>>::encode(owned_blob, buf)
            }

            // Boolean type
            DataKind::Bool(b) => <i64 as Encode<'_, Sqlite>>::encode(*b as i64, buf),

            DataKind::Json(json) => {
                let json_str = serde_json::to_string(json.as_ref())
                    .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync + 'static>)?;
                <String as Encode<'_, Sqlite>>::encode(json_str, buf)
            },

            // UUID type
            DataKind::Uuid(uuid) => <String as Encode<'_, Sqlite>>::encode(uuid.to_string(), buf),
        }
    }
}

impl Type<Sqlite> for DataKind {
    fn type_info() -> <Sqlite as Database>::TypeInfo {
        <str as Type<Sqlite>>::type_info()
    }

    fn compatible(_: &<Sqlite as Database>::TypeInfo) -> bool {
        true
    }
}

impl ValueConvert for DataKind {
    fn convert(value: &dyn Any) -> Self {
        macro_rules! try_convert {
            ($($type:ty => $variant:expr),*) => {
                $(if let Some(v) = unwrap_option::<$type>(value) {
                    return $variant(v);
                })*
                return DataKind::Null;
            };
        }

        try_convert!(
            String => |v: &String| DataKind::Text(v.to_string()),
            &str => |v: &&str| DataKind::Text((*v).to_string()),
            i32 => |v: &i32| DataKind::Integer(*v as i64),            
            u32 => |v: &u32| DataKind::Integer(*v as i64),
            u64 => |v: &u64| DataKind::Integer(*v as i64),
            i64 => |v: &i64| DataKind::Integer(*v),            
            f32 => |v: &f32| DataKind::Real(*v as f64),
            f64 => |v: &f64| DataKind::Real(*v),
            bool => |v: &bool| DataKind::Bool(*v),
            NaiveDateTime => |v: &NaiveDateTime| DataKind::DateTime(*v),
            DateTime<Utc> => |v: &DateTime<Utc>| DataKind::DateTimeUtc(*v),
            NaiveDate => |v: &NaiveDate| DataKind::Date(*v),
            NaiveTime => |v: &NaiveTime| DataKind::Time(*v),
            Vec<u8> => |v: &Vec<u8>| DataKind::Blob(Arc::from(&**v)),
            &[u8] => |v: &&[u8]| DataKind::Blob(Arc::from(*v)),
            Value => |v: &Value| DataKind::Json(Arc::new(v.clone())),
            Uuid => |v: &Uuid| DataKind::Uuid(*v)
        );
    }

    fn is_default_value(value: &Self) -> bool {
        match value {
            DataKind::Integer(v) => *v == 0,
            DataKind::Text(v) => v.is_empty(),
            DataKind::Uuid(v) => v.is_nil(),
            _ => false,
        }
    }
}

// Implement automatic conversion from common types to DataKind
macro_rules! impl_from {
    ($type:ty, $variant:expr) => {
        impl From<$type> for DataKind {
            fn from(item: $type) -> Self {
                $variant(item)
            }
        }
    };
}

// Basic types
impl_from!(String, |value: String| DataKind::Text(value));
impl_from!(&str, |value: &str| DataKind::Text(value.to_string()));
impl_from!(Vec<u8>, |value: Vec<u8>| DataKind::Blob(Arc::from(value)));
impl_from!(&[u8], |value: &[u8]| DataKind::Blob(Arc::from(value)));

// Numeric types
impl_from!(i32, |value: i32| DataKind::Integer(value as i64));
impl_from!(u32, |value: u32| DataKind::Integer(value as i64));
impl_from!(u64, |value: u64| DataKind::Integer(value as i64));
impl_from!(i64, DataKind::Integer);
impl_from!(f32, |value: f32| DataKind::Real(value as f64));
impl_from!(f64, DataKind::Real);

// Boolean type
impl_from!(bool, DataKind::Bool);

// Date and time types
impl_from!(NaiveDateTime, DataKind::DateTime);
impl_from!(DateTime<Utc>, DataKind::DateTimeUtc);
impl_from!(NaiveDate, DataKind::Date);
impl_from!(NaiveTime, DataKind::Time);

// Special types
impl_from!(Value, |value: Value| DataKind::Json(Arc::new(value)));
impl_from!(Uuid, DataKind::Uuid);

impl<'a> From<DataKind> for Cow<'a, DataKind> {
    fn from(value: DataKind) -> Self {
        Cow::Owned(value)
    }
}

impl<'a> From<&'a DataKind> for Cow<'a, DataKind> {
    fn from(value: &'a DataKind) -> Self {
        Cow::Borrowed(value)
    }
}