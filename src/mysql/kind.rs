use std::any::Any;
use std::borrow::Cow;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use sqlx::mysql::{MySql, MySqlTypeInfo};
use sqlx::{Encode, Type, TypeInfo};
use sqlx::types::{Decimal, Uuid};
use serde_json::Value;

use crate::utils::value::{unwrap_option, ValueConvert};

/// Data type enumeration, supporting the main type system of MySQL
#[derive(Default, Debug, Clone)]
pub enum DataKind<'a> {
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
    Text(Cow<'a, str>),   // VARCHAR, CHAR, TEXT

    // Binary types
    Blob(Cow<'a, [u8]>),  // VARBINARY, BINARY, BLOB

    // Time types
    Date(NaiveDate),      // DATE
    Time(NaiveTime),      // TIME (time-of-day only)
    DateTime(NaiveDateTime), // DATETIME
    Timestamp(DateTime<Utc>), // TIMESTAMP

    // Special types
    Json(Value),          // JSON
    Uuid(Uuid),           // BINARY(16), UUID (MariaDB)
    IpAddr(IpAddr),       // INET4/INET6 (MariaDB), VARCHAR/TEXT
    Ipv4Addr(Ipv4Addr),   // INET4 (MariaDB), VARCHAR/TEXT
    Ipv6Addr(Ipv6Addr),   // INET6 (MariaDB), VARCHAR/TEXT
}

impl<'a> Encode<'a, MySql> for DataKind<'a> {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        match self {
            // Basic types
            DataKind::Null => Ok(sqlx::encode::IsNull::Yes),
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
            DataKind::Text(s) => <Cow<'_, str> as Encode<'_, MySql>>::encode(Cow::Borrowed(s), buf),

            // Binary types
            DataKind::Blob(b) => <Vec<u8> as Encode<'_, MySql>>::encode(b.to_vec(), buf),

            // Time types
            DataKind::Date(d) => <NaiveDate as Encode<'_, MySql>>::encode(*d, buf),
            DataKind::Time(t) => <NaiveTime as Encode<'_, MySql>>::encode(*t, buf),
            DataKind::DateTime(dt) => <NaiveDateTime as Encode<'_, MySql>>::encode(*dt, buf),
            DataKind::Timestamp(ts) => <DateTime<Utc> as Encode<'_, MySql>>::encode(*ts, buf),

            // Special types
            DataKind::Json(j) => <Value as Encode<'_, MySql>>::encode(j.clone(), buf),
            DataKind::Uuid(u) => <Uuid as Encode<'_, MySql>>::encode(*u, buf),
            DataKind::IpAddr(ip) => <String as Encode<'_, MySql>>::encode(ip.to_string(), buf),
            DataKind::Ipv4Addr(ipv4) => <String as Encode<'_, MySql>>::encode(ipv4.to_string(), buf),
            DataKind::Ipv6Addr(ipv6) => <String as Encode<'_, MySql>>::encode(ipv6.to_string(), buf),
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
            | "DECIMAL"
            | "CHAR" | "VARCHAR" | "TEXT"
            | "BLOB" | "MEDIUMBLOB" | "LONGBLOB"
            | "DATE" | "DATETIME" | "TIMESTAMP"
            | "JSON" | "NULL"
        )
    }
}

impl<'a> DataKind<'a> {
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
            DataKind::IpAddr(_) => <String as Type<MySql>>::type_info(),
            DataKind::Ipv4Addr(_) => <String as Type<MySql>>::type_info(),
            DataKind::Ipv6Addr(_) => <String as Type<MySql>>::type_info(),
        }
    }
}

impl<'a> ValueConvert<DataKind<'a>> for DataKind<'a> {
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
            String => |v: &String| DataKind::Text(Cow::Owned(v.into())),
            &str => |v: &'a str| DataKind::Text(Cow::Borrowed(v)),
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
            Vec<u8> => |v: &Vec<u8>| DataKind::Blob(Cow::Owned(v.clone())),
            &[u8] => |v: &&'a [u8]| DataKind::Blob(Cow::Borrowed(*v)),
            bool => |v: &bool| DataKind::Bool(*v),
            Value => |v: &Value| DataKind::Json(v.clone()),
            IpAddr => |v: &IpAddr| DataKind::IpAddr(*v),
            Ipv4Addr => |v: &Ipv4Addr| DataKind::Ipv4Addr(*v),
            Ipv6Addr => |v: &Ipv6Addr| DataKind::Ipv6Addr(*v),
            Uuid => |v: &Uuid| DataKind::Uuid(*v)
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

// Basic and numeric types
impl_from!(String, |value: String| DataKind::Text(Cow::Owned(value)));
impl_from!(&'a str, |value: &'a str| DataKind::Text(Cow::Borrowed(value)));
impl_from!(Vec<u8>, |value: Vec<u8>| DataKind::Blob(Cow::Owned(value)));
impl_from!(&'a [u8], |value: &'a [u8]| DataKind::Blob(Cow::Borrowed(value)));
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
impl_from!(Value, DataKind::Json);
impl_from!(Uuid, DataKind::Uuid);
impl_from!(IpAddr, DataKind::IpAddr);
impl_from!(Ipv4Addr, DataKind::Ipv4Addr);
impl_from!(Ipv6Addr, DataKind::Ipv6Addr);