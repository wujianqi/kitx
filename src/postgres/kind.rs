//! Data type definitions and conversions for PostgreSQL database operations.
//! 
//! This module provides the [DataKind] enumeration which represents various database field types
//! supported by PostgreSQL, along with their encoding and type conversion implementations. It handles
//! the mapping between Rust types and PostgreSQL data types, including numeric, string, binary,
//! date/time, network, UUID, and JSON types.
//! 
//! 中文：
//! PostgreSQL 数据库操作的数据类型定义和转换。
//! 
//! 本模块提供了 [DataKind] 枚举，用于表示 PostgreSQL 支持的各种数据库字段类型，
//! 并包含它们的编码和类型转换实现。它处理 Rust 类型和 PostgreSQL 数据类型之间的映射，
//! 包括数值、字符串、二进制、日期/时间、网络、UUID 和 JSON 类型。

use std::any::Any;
use std::error::Error;
use std::sync::Arc;
use std::net::IpAddr;
use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use mac_address::MacAddress;
use sqlx::encode::IsNull;
use sqlx::postgres::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, Postgres};
use sqlx::types::uuid;
use sqlx::{Encode, Type, TypeInfo};
use sqlx::types::{Decimal, ipnetwork::IpNetwork};
use serde_json::Value;
use uuid::Uuid;

use crate::common::conversion::{unwrap_option, ValueConvert};

/// Enum representing PostgreSQL data types, supporting the main PostgreSQL type system
#[derive(Default, Debug, Clone, PartialEq)]
pub enum DataKind {
    // Basic types
    #[default]
    Null,
    Bool(bool),

    // Numeric types
    Int2(i16),              // SMALLINT, SMALLSERIAL, INT2
    Int4(i32),              // INT, SERIAL, INT4
    Int8(i64),              // BIGINT, BIGSERIAL, INT8
    Float4(f32),            // REAL, FLOAT4
    Float8(f64),            // DOUBLE PRECISION, FLOAT8
    Numeric(Decimal),    // NUMERIC

    // String types
    Text(String),     // VARCHAR, CHAR(N), TEXT, NAME, CITEXT

    // Binary types
    Bytea(Arc<[u8]>),   // BYTEA

    // Date and time types
    Date(NaiveDate),        // DATE
    Time(NaiveTime),        // TIME
    Timestamp(NaiveDateTime), // TIMESTAMP
    Timestamptz(DateTime<Utc>), // TIMESTAMPTZ
    Interval(Duration), // INTERVAL

    // Network types
    Inet(IpAddr),        // INET
    Cidr(IpNetwork),        // CIDR
    MacAddr(MacAddress),    // MACADDR

    // UUID type
    Uuid(Uuid),             // UUID

    // JSON types
    Json(Arc<Value>),    // JSON, JSONB
}

impl Encode<'_, Postgres> for DataKind {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, Box<dyn Error + Send + Sync>> {
        match self {
            DataKind::Null => Ok(IsNull::Yes),
            DataKind::Bool(b) => <bool as Encode<'_, Postgres>>::encode(*b, buf),
            DataKind::Int2(i) => <i16 as Encode<'_, Postgres>>::encode(*i, buf),
            DataKind::Int4(i) => <i32 as Encode<'_, Postgres>>::encode(*i, buf),
            DataKind::Int8(i) => <i64 as Encode<'_, Postgres>>::encode(*i, buf),
            DataKind::Float4(f) => <f32 as Encode<'_, Postgres>>::encode(*f, buf),
            DataKind::Float8(d) => <f64 as Encode<'_, Postgres>>::encode(*d, buf),
            DataKind::Numeric(n) => <Decimal as Encode<'_, Postgres>>::encode(*n, buf),
            DataKind::Text(s) => <String as Encode<'_, Postgres>>::encode(s.to_string(), buf),
            DataKind::Bytea(b) => <&[u8] as Encode<'_, Postgres>>::encode(b, buf),
            DataKind::Date(d) => <NaiveDate as Encode<'_, Postgres>>::encode(*d, buf),
            DataKind::Time(t) => <NaiveTime as Encode<'_, Postgres>>::encode(*t, buf),
            DataKind::Timestamp(ts) => <NaiveDateTime as Encode<'_, Postgres>>::encode(*ts, buf),
            DataKind::Timestamptz(tstz) => <DateTime<Utc> as Encode<'_, Postgres>>::encode(*tstz, buf),
            DataKind::Interval(i) => <Duration as Encode<'_, Postgres>>::encode(*i, buf),
            DataKind::Inet(ip) => <IpAddr as Encode<'_, Postgres>>::encode(*ip, buf),
            DataKind::Cidr(cidr) => <IpNetwork as Encode<'_, Postgres>>::encode(*cidr, buf),
            DataKind::MacAddr(mac) => <[u8; 6] as Encode<'_, Postgres>>::encode(mac.bytes(), buf),
            DataKind::Uuid(uuid) => <Uuid as Encode<'_, Postgres>>::encode(*uuid, buf),
            DataKind::Json(j) => <&Value as Encode<'_, Postgres>>::encode(j, buf),            
        }
    }

    fn produces(&self) -> Option<PgTypeInfo> {
        Some(self.get_type_info())
    }
}

impl Type<Postgres> for DataKind {
    fn type_info() -> PgTypeInfo {
        <str as Type<Postgres>>::type_info()
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        matches!(
            ty.name(),
            "BOOL" | "INT2" | "INT4" | "INT8" | "FLOAT4" | "FLOAT8" | "NUMERIC"
            | "TEXT" | "BYTEA"
            | "DATE" | "TIME" | "TIMESTAMP" | "TIMESTAMPTZ" | "INTERVAL"
            | "INET" | "CIDR" | "MACADDR" | "UUID"
            | "JSON" | "JSONB" | "NULL"
        )
    }
}

impl DataKind {
    pub fn get_type_info(&self) -> PgTypeInfo {
        match self {
            DataKind::Bool(_) => <bool as Type<Postgres>>::type_info(),
            DataKind::Int2(_) => <i16 as Type<Postgres>>::type_info(),
            DataKind::Int4(_) => <i32 as Type<Postgres>>::type_info(),
            DataKind::Int8(_) => <i64 as Type<Postgres>>::type_info(),
            DataKind::Float4(_) => <f32 as Type<Postgres>>::type_info(),
            DataKind::Float8(_) => <f64 as Type<Postgres>>::type_info(),
            DataKind::Numeric(_) => <Decimal as Type<Postgres>>::type_info(),
            DataKind::Text(_) => <str as Type<Postgres>>::type_info(),
            DataKind::Bytea(_) => <Vec<u8> as Type<Postgres>>::type_info(),
            DataKind::Date(_) => <NaiveDate as Type<Postgres>>::type_info(),
            DataKind::Time(_) => <NaiveTime as Type<Postgres>>::type_info(),
            DataKind::Timestamp(_) => <NaiveDateTime as Type<Postgres>>::type_info(),
            DataKind::Timestamptz(_) => <DateTime<Utc> as Type<Postgres>>::type_info(),
            DataKind::Interval(_) => <Duration as Type<Postgres>>::type_info(),
            DataKind::Inet(_) => <IpAddr as Type<Postgres>>::type_info(),
            DataKind::Cidr(_) => <IpNetwork as Type<Postgres>>::type_info(),
            DataKind::MacAddr(_) => <[u8; 6] as Type<Postgres>>::type_info(),
            DataKind::Uuid(_) => <Uuid as Type<Postgres>>::type_info(),
            DataKind::Json(_) => <Value as Type<Postgres>>::type_info(),
            DataKind::Null => <str as Type<Postgres>>::type_info(),
        }
    }
}

impl PgHasArrayType for DataKind {
    fn array_type_info() -> PgTypeInfo {
        <Vec<DataKind> as Type<Postgres>>::type_info()
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
            &str => |v: &&str| DataKind::Text(v.to_string()),
            i8 => |v: &i8| DataKind::Int2(*v as i16),
            i16 => |v: &i16| DataKind::Int2(*v),
            i32 => |v: &i32| DataKind::Int4(*v),
            i64 => |v: &i64| DataKind::Int8(*v),
            f32 => |v: &f32| DataKind::Float4(*v),
            f64 => |v: &f64| DataKind::Float8(*v),
            Decimal => |v: &Decimal| DataKind::Numeric(*v),
            NaiveDate => |v: &NaiveDate| DataKind::Date(*v),
            NaiveTime => |v: &NaiveTime| DataKind::Time(*v),
            NaiveDateTime => |v: &NaiveDateTime| DataKind::Timestamp(*v),
            DateTime<Utc> => |v: &DateTime<Utc>| DataKind::Timestamptz(*v),
            Duration => |v: &Duration| DataKind::Interval(*v),
            Vec<u8> => |v: &Vec<u8>| DataKind::Bytea(Arc::from(v.as_slice())),
            &[u8] => |v: &&[u8]| DataKind::Bytea(Arc::from(*v)),
            bool => |v: &bool| DataKind::Bool(*v),
            Uuid => |v: &Uuid| DataKind::Uuid(*v),
            Value => |v: &Value| DataKind::Json(Arc::new(v.clone())),
            IpAddr => |v: &IpAddr| DataKind::Inet(*v),
            IpNetwork => |v: &IpNetwork| DataKind::Cidr(*v),
            MacAddress => |v: &MacAddress| DataKind::MacAddr(*v)
        );
    }

    fn is_default_value(value: &Self) -> bool {
        match value {
            DataKind::Int2(v) => *v == 0,
            DataKind::Int4(v) => *v == 0,
            DataKind::Int8(v) => *v == 0,
            DataKind::Uuid(v) => v.is_nil(),
            DataKind::Text(v) => v.is_empty(),
            _ => false,
        }
    }
}

macro_rules! impl_from {
    ($type:ty, $variant:expr) => {
        impl From<$type> for DataKind {
            fn from(item: $type) -> Self {
                $variant(item)
            }
        }
    };
}

impl_from!(String, DataKind::Text);
impl_from!(&str, |value: &str| DataKind::Text(value.to_string()));
impl_from!(Vec<u8>, |value: Vec<u8>| DataKind::Bytea(Arc::from(value)));
impl_from!(&[u8], |value: &[u8]| DataKind::Bytea(Arc::from(value)));
impl_from!(i8, |value: i8| DataKind::Int2(value as i16));
impl_from!(u8, |value: u8| DataKind::Int2(value as i16));
impl_from!(i16, DataKind::Int2);
impl_from!(u16, |value: u16| DataKind::Int2(value as i16));
impl_from!(i32, DataKind::Int4);
impl_from!(u32, |value: u32| DataKind::Int4(value as i32));
impl_from!(i64, DataKind::Int8);
impl_from!(u64, |value: u64| DataKind::Int8(value as i64));
impl_from!(f32, DataKind::Float4);
impl_from!(f64, DataKind::Float8);
impl_from!(bool, DataKind::Bool);
impl_from!(NaiveDate, DataKind::Date);
impl_from!(NaiveTime, DataKind::Time);
impl_from!(NaiveDateTime, DataKind::Timestamp);
impl_from!(DateTime<Utc>, DataKind::Timestamptz);
impl_from!(Duration, DataKind::Interval);
impl_from!(Value, |value: Value| DataKind::Json(Arc::new(value)));
impl_from!(Uuid, DataKind::Uuid);
impl_from!(Decimal, DataKind::Numeric);
impl_from!(IpAddr, DataKind::Inet);
impl_from!(IpNetwork, DataKind::Cidr);
impl_from!(MacAddress, DataKind::MacAddr);
