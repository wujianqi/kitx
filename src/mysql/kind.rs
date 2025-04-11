use std::any::Any;
use std::borrow::Cow;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use sqlx::mysql::{MySql, MySqlTypeInfo};
use sqlx::{Encode, Type, TypeInfo};
use serde_json::Value;

use crate::utils::value::{unwrap_option, ValueConvert};

/// Data type enumeration, supporting the main type system of MySQL
#[derive(Debug, Clone)]
pub enum DataKind<'a> {
    // Basic types
    Null,
    Bool(bool),

    // Numeric types
    TinyInt(i8),
    SmallInt(i16),
    Int(i32),
    BigInt(i64),
    Float(f32),
    Double(f64),

    // String types
    Text(Cow<'a, str>),

    // Binary types
    Blob(Cow<'a, [u8]>),

    // Time types
    Date(NaiveDate),
    Time(NaiveTime),
    DateTime(NaiveDateTime),
    Timestamp(DateTime<Utc>),

    // Special types
    Json(Value),
}

impl<'a> Encode<'a, MySql> for DataKind<'a> {
    /// Encodes the `DataKind` value into a MySQL-compatible format.
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        match self {
            DataKind::Null => Ok(sqlx::encode::IsNull::Yes),
            DataKind::Bool(b) => <bool as Encode<'_, MySql>>::encode(*b, buf),
            DataKind::TinyInt(i) => <i8 as Encode<'_, MySql>>::encode(*i, buf),
            DataKind::SmallInt(i) => <i16 as Encode<'_, MySql>>::encode(*i, buf),
            DataKind::Int(i) => <i32 as Encode<'_, MySql>>::encode(*i, buf),
            DataKind::BigInt(i) => <i64 as Encode<'_, MySql>>::encode(*i, buf),
            DataKind::Float(f) => <f32 as Encode<'_, MySql>>::encode(*f, buf),
            DataKind::Double(d) => <f64 as Encode<'_, MySql>>::encode(*d, buf),
            DataKind::Text(s) => <Cow<'_, str> as Encode<'_, MySql>>::encode(Cow::Borrowed(s), buf),
            DataKind::Blob(b) => <Vec<u8> as Encode<'_, MySql>>::encode(b.to_vec(), buf),
            DataKind::Date(d) => <NaiveDate as Encode<'_, MySql>>::encode(*d, buf),
            DataKind::Time(t) => <NaiveTime as Encode<'_, MySql>>::encode(*t, buf),
            DataKind::DateTime(dt) => <NaiveDateTime as Encode<'_, MySql>>::encode(*dt, buf),
            DataKind::Timestamp(ts) => <DateTime<Utc> as Encode<'_, MySql>>::encode(*ts, buf),
            DataKind::Json(j) => <Value as Encode<'_, MySql>>::encode(j.clone(), buf),
        }
    }

    fn produces(&self) -> Option<MySqlTypeInfo> {
        Some(self.get_type_info())
    }
}

impl<'a> Type<MySql> for DataKind<'a> {
    fn type_info() -> MySqlTypeInfo {
        <str as Type<MySql>>::type_info()
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        matches!(
            ty.name(),
            "TINYINT" | "SMALLINT" | "INT" | "BIGINT" 
            | "FLOAT" | "DOUBLE" 
            | "CHAR" | "VARCHAR" | "TEXT" 
            | "DATE" | "DATETIME" | "TIMESTAMP" 
            | "BLOB" | "MEDIUMBLOB" | "LONGBLOB" 
            | "JSON" | "NULL"
        )
    }
}

impl<'a> DataKind<'a> {
    pub fn get_type_info(&self) -> MySqlTypeInfo {
        match self {
            DataKind::Text(_) => <str as Type<MySql>>::type_info(),
            DataKind::TinyInt(_) => <i8 as Type<MySql>>::type_info(),
            DataKind::SmallInt(_) => <i16 as Type<MySql>>::type_info(),
            DataKind::Int(_) => <i32 as Type<MySql>>::type_info(),
            DataKind::BigInt(_) => <i64 as Type<MySql>>::type_info(),
            DataKind::Float(_) => <f32 as Type<MySql>>::type_info(),
            DataKind::Double(_) => <f64 as Type<MySql>>::type_info(),
            DataKind::Date(_) => <NaiveDate as Type<MySql>>::type_info(),
            DataKind::Time(_) => <NaiveTime as Type<MySql>>::type_info(),
            DataKind::DateTime(_) => <NaiveDateTime as Type<MySql>>::type_info(),
            DataKind::Timestamp(_) => <DateTime<Utc> as Type<MySql>>::type_info(),
            DataKind::Blob(_) => <Vec<u8> as Type<MySql>>::type_info(),
            DataKind::Bool(_) => <bool as Type<MySql>>::type_info(),
            DataKind::Json(_) => <Value as Type<MySql>>::type_info(),
            DataKind::Null => <str as Type<MySql>>::type_info(),
        }
    }
}

impl<'a> ValueConvert<DataKind<'a>> for DataKind<'a> {    
    /// Convert any type of value to the `DataKind` enum type.
    fn convert(value: &dyn Any) -> DataKind<'a> {
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
            i8 => |v: &i8| DataKind::TinyInt(*v),
            i16 => |v: &i16| DataKind::SmallInt(*v),
            i32 => |v: &i32| DataKind::Int(*v),
            i64 => |v: &i64| DataKind::BigInt(*v),
            f32 => |v: &f32| DataKind::Float(*v),
            f64 => |v: &f64| DataKind::Double(*v),
            NaiveDate => |v: &NaiveDate| DataKind::Date(*v),
            NaiveTime => |v: &NaiveTime| DataKind::Time(*v),
            NaiveDateTime => |v: &NaiveDateTime| DataKind::DateTime(*v),
            DateTime<Utc> => |v: &DateTime<Utc>| DataKind::Timestamp(*v),
            Vec<u8> => |v: &Vec<u8>| DataKind::Blob(Cow::Owned(v.clone())),
            &[u8] => |v: &&'a [u8]| DataKind::Blob(Cow::Borrowed(*v)),
            bool => |v: &bool| DataKind::Bool(*v),
            Value => |v: &Value| DataKind::Json(v.clone())
        );
    }
}

// Implement From trait for common types
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
impl_from!(Vec<u8>, |value: Vec<u8>| DataKind::Blob(Cow::Owned(value)));
impl_from!(&'a [u8], |value: &'a [u8]| DataKind::Blob(Cow::Borrowed(value)));
impl_from!(i8, DataKind::TinyInt);
impl_from!(i16, DataKind::SmallInt);
impl_from!(i32, DataKind::Int);
impl_from!(i64, DataKind::BigInt);
impl_from!(u64, |value: u64| DataKind::BigInt(value as i64));
impl_from!(f32, DataKind::Float);
impl_from!(f64, DataKind::Double);
impl_from!(bool, DataKind::Bool);
impl_from!(NaiveDate, DataKind::Date);
impl_from!(NaiveTime, DataKind::Time);
impl_from!(NaiveDateTime, DataKind::DateTime);
impl_from!(DateTime<Utc>, DataKind::Timestamp);
impl_from!(Value, DataKind::Json);