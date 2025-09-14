//! Data type definitions and conversions for MySQL and MariaDB database operations.
//! 
//! This module provides the [DataKind] enumeration which represents various database field types
//! supported by MySQL and MariaDB, along with their encoding and type conversion implementations. 
//! It handles the mapping between Rust types and MySQL/MariaDB data types, including numeric, 
//! string, binary, temporal, JSON, UUID, and IP address types.
//! 
//! MySQL 和 MariaDB 数据库操作的数据类型定义和转换。
//! 
//! 本模块提供了 [DataKind] 枚举，用于表示 MySQL 和 MariaDB 支持的各种数据库字段类型，
//! 并包含它们的编码和类型转换实现。它处理 Rust 类型和 MySQL/MariaDB 数据类型之间的映射，
//! 包括数值、字符串、二进制、时间、JSON、UUID 和 IP 地址类型。

use std::borrow::Cow;
use std::error::Error;
use std::sync::Arc;
use std::any::Any;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use sqlx::encode::IsNull;
use sqlx::mysql::{MySql, MySqlTypeInfo};
use sqlx::{Encode, Type, TypeInfo};
use sqlx::types::{Decimal, Uuid};
use serde_json::Value;

use crate::common::conversion::{unwrap_option, ValueConvert};

/// Enum representing PostgreSQL data types, supporting the main PostgreSQL type system
#[derive(Default, Debug, Clone, PartialEq)]
pub enum DataKind {
    // Basic types
    #[default]
    Null,
    Bool(bool), // TINYINT(1), BOOLEAN, BOOL

    // Numeric types
    TinyInt(i8),          // TINYINT
    SmallInt(i16),        // SMALLINT
    Int(i32),             // INT
    BigInt(i64),          // BIGINT
    UnsignedTinyInt(u8),  // TINYINT UNSIGNED
    UnsignedSmallInt(u16),// SMALLINT UNSIGNED
    UnsignedInt(u32),     // INT UNSIGNED
    UnsignedBigInt(u64),  // BIGINT UNSIGNED
    Float(f32),           // FLOAT
    Double(f64),          // DOUBLE

    // Decimal types
    Decimal(Decimal),  // DECIMAL

    // String types
    Text(String),   // VARCHAR, CHAR, TEXT

    // Binary types
    Blob(Arc<[u8]>),  // VARBINARY, BINARY, BLOB

    // Time types
    Date(NaiveDate),      // DATE
    Time(NaiveTime),      // TIME (time-of-day only)
    DateTime(NaiveDateTime), // DATETIME
    Timestamp(DateTime<Utc>), // TIMESTAMP

    // Special types
    Json(Arc<Value>),          // JSON (both MySQL 5.7+ and MariaDB 10.2+)
    
    // UUID support - stored as BINARY(16) in both MySQL and MariaDB
    // Note: MariaDB 10.7+ has native UUID type, but sqlx uses BINARY(16) for compatibility
    Uuid(Uuid),
    
    // IP Address types - stored as string for maximum compatibility
    // Note: While MariaDB has native INET4/INET6 types, they're not yet fully supported by sqlx
    IpAddr(IpAddr),       // Stored as VARCHAR for compatibility
    Ipv4Addr(Ipv4Addr),   // Stored as VARCHAR or can be optimized to INT UNSIGNED
    Ipv6Addr(Ipv6Addr),   // Stored as VARCHAR or BINARY(16)
}

impl Encode<'_, MySql> for DataKind {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> Result<IsNull, Box<dyn Error + Send + Sync>> {
        match self {
            // Basic types
            DataKind::Null => Ok(IsNull::Yes),
            DataKind::Bool(b) => <bool as Encode<'_, MySql>>::encode(*b, buf),

            // Numeric types
            DataKind::TinyInt(i) => <i8 as Encode<'_, MySql>>::encode(*i, buf),
            DataKind::SmallInt(i) => <i16 as Encode<'_, MySql>>::encode(*i, buf),
            DataKind::Int(i) => <i32 as Encode<'_, MySql>>::encode(*i, buf),
            DataKind::BigInt(i) => <i64 as Encode<'_, MySql>>::encode(*i, buf),
            DataKind::UnsignedTinyInt(i) => <u8 as Encode<'_, MySql>>::encode(*i, buf),
            DataKind::UnsignedSmallInt(i) => <u16 as Encode<'_, MySql>>::encode(*i, buf),
            DataKind::UnsignedInt(i) => <u32 as Encode<'_, MySql>>::encode(*i, buf),
            DataKind::UnsignedBigInt(i) => <u64 as Encode<'_, MySql>>::encode(*i, buf),
            DataKind::Float(f) => <f32 as Encode<'_, MySql>>::encode(*f, buf),
            DataKind::Double(d) => <f64 as Encode<'_, MySql>>::encode(*d, buf),

            // Decimal types
            DataKind::Decimal(d) => <Decimal as Encode<'_, MySql>>::encode(*d, buf),

            // String types
            DataKind::Text(s) => <String as Encode<'_, MySql>>::encode(s.to_string(), buf),

            // Binary types
            DataKind::Blob(blob) => <Vec<u8> as Encode<'_, MySql>>::encode(blob.to_vec(), buf),

            // Time types
            DataKind::Date(d) => <NaiveDate as Encode<'_, MySql>>::encode(*d, buf),
            DataKind::Time(t) => <NaiveTime as Encode<'_, MySql>>::encode(*t, buf),
            DataKind::DateTime(dt) => <NaiveDateTime as Encode<'_, MySql>>::encode(*dt, buf),
            DataKind::Timestamp(ts) => <DateTime<Utc> as Encode<'_, MySql>>::encode(*ts, buf),

            // Special types
            DataKind::Json(json) => {
                let owned_json = Arc::clone(&json);
                <Value as Encode<'_, MySql>>::encode(Arc::try_unwrap(owned_json)
                    .unwrap_or_else(|arc| (*arc).clone()), buf)
            },
            
            // UUID - encoded as BINARY(16) for both MySQL and MariaDB
            DataKind::Uuid(u) => <Uuid as Encode<'_, MySql>>::encode(*u, buf),
            
            // IP Address - stored as string for compatibility
            DataKind::IpAddr(ip) => <String as Encode<'_, MySql>>::encode(ip.to_string(), buf),
            DataKind::Ipv4Addr(ipv4) => <String as Encode<'_, MySql>>::encode(ipv4.to_string(), buf),
            DataKind::Ipv6Addr(ipv6) => <String as Encode<'_, MySql>>::encode(ipv6.to_string(), buf),
        }
    }

    fn produces(&self) -> Option<MySqlTypeInfo> {
        Some(self.get_type_info())
    }
}

impl Type<MySql> for DataKind {
    fn type_info() -> MySqlTypeInfo {
        <str as Type<MySql>>::type_info()
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        matches!(
            ty.name(),
            // Standard MySQL/MariaDB types
            "TINYINT" | "SMALLINT" | "INT" | "BIGINT"
            | "FLOAT" | "DOUBLE" | "DECIMAL"
            | "CHAR" | "VARCHAR" | "TEXT" | "LONGTEXT"
            | "BINARY" | "VARBINARY" | "BLOB" | "MEDIUMBLOB" | "LONGBLOB"
            | "DATE" | "TIME" | "DATETIME" | "TIMESTAMP"
            | "JSON" | "NULL" | "BOOLEAN" | "BOOL"
            // UUID support (BINARY for MySQL, UUID for MariaDB)
            | "UUID"
        )
    }
}

impl DataKind {
    pub fn get_type_info(&self) -> MySqlTypeInfo {
        match self {
            // Basic types
            DataKind::Null => <str as Type<MySql>>::type_info(),
            DataKind::Bool(_) => <bool as Type<MySql>>::type_info(),

            // Numeric types
            DataKind::TinyInt(_) => <i8 as Type<MySql>>::type_info(),
            DataKind::SmallInt(_) => <i16 as Type<MySql>>::type_info(),
            DataKind::Int(_) => <i32 as Type<MySql>>::type_info(),
            DataKind::BigInt(_) => <i64 as Type<MySql>>::type_info(),
            DataKind::UnsignedTinyInt(_) => <u8 as Type<MySql>>::type_info(),
            DataKind::UnsignedSmallInt(_) => <u16 as Type<MySql>>::type_info(),
            DataKind::UnsignedInt(_) => <u32 as Type<MySql>>::type_info(),
            DataKind::UnsignedBigInt(_) => <u64 as Type<MySql>>::type_info(),
            DataKind::Float(_) => <f32 as Type<MySql>>::type_info(),
            DataKind::Double(_) => <f64 as Type<MySql>>::type_info(),

            // Decimal types
            DataKind::Decimal(_) => <Decimal as Type<MySql>>::type_info(),

            // String types
            DataKind::Text(_) => <str as Type<MySql>>::type_info(),

            // Binary types
            DataKind::Blob(_) => <Vec<u8> as Type<MySql>>::type_info(),

            // Time types
            DataKind::Date(_) => <NaiveDate as Type<MySql>>::type_info(),
            DataKind::Time(_) => <NaiveTime as Type<MySql>>::type_info(),
            DataKind::DateTime(_) => <NaiveDateTime as Type<MySql>>::type_info(),
            DataKind::Timestamp(_) => <DateTime<Utc> as Type<MySql>>::type_info(),

            // Special types
            DataKind::Json(_) => <Value as Type<MySql>>::type_info(),
            DataKind::Uuid(_) => <Uuid as Type<MySql>>::type_info(),
            
            // IP Address types - all use string type info for compatibility
            DataKind::IpAddr(_) => <String as Type<MySql>>::type_info(),
            DataKind::Ipv4Addr(_) => <String as Type<MySql>>::type_info(),
            DataKind::Ipv6Addr(_) => <String as Type<MySql>>::type_info(),
        }
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
            String => |v: &String| DataKind::Text(v.clone()),
            &str => |v: &&str| DataKind::Text(v.to_string()),
            i8 => |v: &i8| DataKind::TinyInt(*v),
            i16 => |v: &i16| DataKind::SmallInt(*v),
            i32 => |v: &i32| DataKind::Int(*v),
            i64 => |v: &i64| DataKind::BigInt(*v),
            u8 => |v: &u8| DataKind::UnsignedTinyInt(*v),
            u16 => |v: &u16| DataKind::UnsignedSmallInt(*v),
            u32 => |v: &u32| DataKind::UnsignedInt(*v),
            u64 => |v: &u64| DataKind::UnsignedBigInt(*v),
            f32 => |v: &f32| DataKind::Float(*v),
            f64 => |v: &f64| DataKind::Double(*v),
            NaiveDate => |v: &NaiveDate| DataKind::Date(*v),
            NaiveTime => |v: &NaiveTime| DataKind::Time(*v),
            NaiveDateTime => |v: &NaiveDateTime| DataKind::DateTime(*v),
            DateTime<Utc> => |v: &DateTime<Utc>| DataKind::Timestamp(*v),
            Vec<u8> => |v: &Vec<u8>| DataKind::Blob(Arc::from(v.as_slice())),
            &[u8] => |v: &&[u8]| DataKind::Blob(Arc::from(*v)),
            bool => |v: &bool| DataKind::Bool(*v),
            Value => |v: &Value| DataKind::Json(Arc::new(v.clone())),
            Uuid => |v: &Uuid| DataKind::Uuid(*v),
            IpAddr => |v: &IpAddr| DataKind::IpAddr(*v),
            Ipv4Addr => |v: &Ipv4Addr| DataKind::Ipv4Addr(*v),
            Ipv6Addr => |v: &Ipv6Addr| DataKind::Ipv6Addr(*v)
        );
    }

    fn is_default_value(value: &Self) -> bool {
        match value {
            // 常用作主键的类型
            DataKind::Int(v) => *v == 0,
            DataKind::BigInt(v) => *v == 0,
            DataKind::UnsignedInt(v) => *v == 0,
            DataKind::UnsignedBigInt(v) => *v == 0,
            DataKind::Uuid(v) => v.is_nil(),
            DataKind::Text(v) => v.is_empty(),
            _ => false,
        }
    }
}

// Implement From trait for common types
macro_rules! impl_from {
    ($type:ty, $variant:expr) => {
        impl From<$type> for DataKind {
            fn from(item: $type) -> Self {
                $variant(item)
            }
        }
    };
}

// Basic and numeric types
impl_from!(String, |value: String| DataKind::Text(value));
impl_from!(&str, |value: &str| DataKind::Text(value.to_string()));
impl_from!(Vec<u8>, |value: Vec<u8>| DataKind::Blob(Arc::from(value)));
impl_from!(&[u8], |value: &[u8]| DataKind::Blob(Arc::from(value)));
impl_from!(i8, DataKind::TinyInt);
impl_from!(i16, DataKind::SmallInt);
impl_from!(i32, DataKind::Int);
impl_from!(i64, DataKind::BigInt);
impl_from!(u8, DataKind::UnsignedTinyInt);
impl_from!(u16, DataKind::UnsignedSmallInt);
impl_from!(u32, DataKind::UnsignedInt);
impl_from!(u64, DataKind::UnsignedBigInt);
impl_from!(f32, DataKind::Float);
impl_from!(f64, DataKind::Double);
impl_from!(bool, DataKind::Bool);

// Time types
impl_from!(NaiveDate, DataKind::Date);
impl_from!(NaiveTime, DataKind::Time);
impl_from!(NaiveDateTime, DataKind::DateTime);
impl_from!(DateTime<Utc>, DataKind::Timestamp);

// Special types
impl_from!(Value, |value: Value| DataKind::Json(Arc::new(value)));
impl_from!(Uuid, DataKind::Uuid);
impl_from!(IpAddr, DataKind::IpAddr);
impl_from!(Ipv4Addr, DataKind::Ipv4Addr);
impl_from!(Ipv6Addr, DataKind::Ipv6Addr);


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