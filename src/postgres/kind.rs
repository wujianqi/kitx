use std::any::Any;
use std::borrow::Cow;
use std::error::Error;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use sqlx::encode::IsNull;
use sqlx::postgres::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, Postgres};
use sqlx::types::uuid;
use sqlx::{Encode, Type, TypeInfo};
use sqlx::types::Decimal;
use serde_json::Value;
use uuid::Uuid;

use crate::common::util::unwrap_option;

/// Enum representing PostgreSQL data types, supporting the main PostgreSQL type system
#[derive(Debug, Clone)]
pub enum DataKind<'a> {
    // Basic types
    Null,
    Bool(bool),

    // Numeric types
    Int2(i16),
    Int4(i32),
    Int8(i64),
    Float4(f32),
    Float8(f64),
    Numeric(Decimal),

    // String types
    Text(Cow<'a, str>),

    // Binary types
    Bytea(Cow<'a, [u8]>),

    // Date and time types
    Date(NaiveDate),
    Time(NaiveTime),
    Timestamp(NaiveDateTime),
    Timestamptz(DateTime<Utc>),

    // Network types
    Inet(String),
    Cidr(String),
    MacAddr([u8; 6]),

    // UUID type
    Uuid(Uuid),

    // JSON types
    Json(Value),
    Jsonb(Value),

    // Array type
    Array(Vec<DataKind<'a>>),
}

impl<'a> Encode<'a, Postgres> for DataKind<'a> {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, Box<dyn Error + Send + Sync>> {
        match self {
            DataKind::Null => Ok(IsNull::Yes),
            DataKind::Bool(b) => <bool as Encode<'_, Postgres>>::encode(*b, buf),
            DataKind::Int2(i) => <i16 as Encode<'_, Postgres>>::encode(*i, buf),
            DataKind::Int4(i) => <i32 as Encode<'_, Postgres>>::encode(*i, buf),
            DataKind::Int8(i) => <i64 as Encode<'_, Postgres>>::encode(*i, buf),
            DataKind::Float4(f) => <f32 as Encode<'_, Postgres>>::encode(*f, buf),
            DataKind::Float8(d) => <f64 as Encode<'_, Postgres>>::encode(*d, buf),
            DataKind::Numeric(n) => <Decimal as Encode<'_, Postgres>>::encode(n.clone(), buf),
            DataKind::Text(s) => <Cow<'_, str> as Encode<'_, Postgres>>::encode(Cow::Borrowed(s), buf),
            DataKind::Bytea(b) => <Vec<u8> as Encode<'_, Postgres>>::encode(b.to_vec(), buf),
            DataKind::Date(d) => <NaiveDate as Encode<'_, Postgres>>::encode(*d, buf),
            DataKind::Time(t) => <NaiveTime as Encode<'_, Postgres>>::encode(*t, buf),
            DataKind::Timestamp(ts) => <NaiveDateTime as Encode<'_, Postgres>>::encode(*ts, buf),
            DataKind::Timestamptz(tstz) => <DateTime<Utc> as Encode<'_, Postgres>>::encode(*tstz, buf),
            DataKind::Inet(ip) => <String as Encode<'_, Postgres>>::encode(ip.clone(), buf),
            DataKind::Cidr(cidr) => <String as Encode<'_, Postgres>>::encode(cidr.clone(), buf),
            DataKind::MacAddr(mac) => <[u8; 6] as Encode<'_, Postgres>>::encode(*mac, buf),
            DataKind::Uuid(uuid) => <Uuid as Encode<'_, Postgres>>::encode(*uuid, buf),
            DataKind::Json(j) => <Value as Encode<'_, Postgres>>::encode(j.clone(), buf),
            DataKind::Jsonb(j) => <Value as Encode<'_, Postgres>>::encode(j.clone(), buf),
            DataKind::Array(arr) => {
                for item in arr {
                    let _ = item.encode_by_ref(buf)?;
                }
                Ok(IsNull::No)
            },
        }
    }

    fn produces(&self) -> Option<PgTypeInfo> {
        Some(self.get_type_info())
    }
}

impl<'a> Type<Postgres> for DataKind<'a> {
    fn type_info() -> PgTypeInfo {
        <str as Type<Postgres>>::type_info()
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        matches!(
            ty.name(),
            "BOOL" | "INT2" | "INT4" | "INT8" | "FLOAT4" | "FLOAT8" | "NUMERIC"
            | "TEXT" | "BYTEA"
            | "DATE" | "TIME" | "TIMESTAMP" | "TIMESTAMPTZ"
            | "INET" | "CIDR" | "MACADDR" | "UUID"
            | "JSON" | "JSONB" | "ARRAY" | "NULL"
        )
    }
}

impl<'a> DataKind<'a> {
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
            DataKind::Inet(_) => <String as Type<Postgres>>::type_info(),
            DataKind::Cidr(_) => <String as Type<Postgres>>::type_info(),
            DataKind::MacAddr(_) => <[u8; 6] as Type<Postgres>>::type_info(),
            DataKind::Uuid(_) => <Uuid as Type<Postgres>>::type_info(),
            DataKind::Json(_) => <Value as Type<Postgres>>::type_info(),
            DataKind::Jsonb(_) => <Value as Type<Postgres>>::type_info(),
            DataKind::Array(_) => <Vec<DataKind> as Type<Postgres>>::type_info(),
            DataKind::Null => <str as Type<Postgres>>::type_info(),
        }
    }
}

impl PgHasArrayType for DataKind<'_> {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_DataKind")
    }
}

/// Converts a value of any type to the `DataKind` enum
pub fn value_convert<'a>(value: &dyn Any) -> DataKind<'a> {
    macro_rules! try_convert {
        ($($type:ty => $variant:expr),*) => {
            $(if let Some(v) = unwrap_option::<$type>(value) {
                return $variant(v);
            })*
            return DataKind::Null;
        };
    }

    try_convert!(
        String => |v: &String| DataKind::Text(Cow::Owned(v.into())),
        &str => |v: &'a str| DataKind::Text(Cow::Borrowed(v)),
        i16 => |v: &i16| DataKind::Int2(*v),
        i32 => |v: &i32| DataKind::Int4(*v),
        i64 => |v: &i64| DataKind::Int8(*v),
        f32 => |v: &f32| DataKind::Float4(*v),
        f64 => |v: &f64| DataKind::Float8(*v),
        NaiveDate => |v: &NaiveDate| DataKind::Date(*v),
        NaiveTime => |v: &NaiveTime| DataKind::Time(*v),
        NaiveDateTime => |v: &NaiveDateTime| DataKind::Timestamp(*v),
        DateTime<Utc> => |v: &DateTime<Utc>| DataKind::Timestamptz(*v),
        Vec<u8> => |v: &Vec<u8>| DataKind::Bytea(Cow::Owned(v.clone())),
        &[u8] => |v: &&'a [u8]| DataKind::Bytea(Cow::Borrowed(*v)),
        bool => |v: &bool| DataKind::Bool(*v),
        Uuid => |v: &Uuid| DataKind::Uuid(*v),
        Decimal => |v: &Decimal| DataKind::Numeric(*v),
        Value => |v: &Value| DataKind::Json(v.clone())
    );
}

macro_rules! impl_from {
    ($type:ty, $variant:expr) => {
        impl<'a> From<$type> for DataKind<'a> {
            fn from(item: $type) -> Self {
                $variant(item)
            }
        }
    };
}

impl_from!(String, |value: String| DataKind::Text(Cow::Owned(value)));
impl_from!(&'a str, |value: &'a str| DataKind::Text(Cow::Borrowed(value)));
impl_from!(Vec<u8>, |value: Vec<u8>| DataKind::Bytea(Cow::Owned(value)));
impl_from!(&'a [u8], |value: &'a [u8]| DataKind::Bytea(Cow::Borrowed(value)));
impl_from!(i16, DataKind::Int2);
impl_from!(i32, DataKind::Int4);
impl_from!(i64, DataKind::Int8);
impl_from!(f32, DataKind::Float4);
impl_from!(f64, DataKind::Float8);
impl_from!(bool, DataKind::Bool);
impl_from!(NaiveDate, DataKind::Date);
impl_from!(NaiveTime, DataKind::Time);
impl_from!(NaiveDateTime, DataKind::Timestamp);
impl_from!(DateTime<Utc>, DataKind::Timestamptz);
impl_from!(Value, DataKind::Json);
impl_from!(Uuid, DataKind::Uuid);
impl_from!(Decimal, DataKind::Numeric);
impl_from!(Vec<DataKind<'a>>, DataKind::Array);
