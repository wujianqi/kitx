use std::any::Any;
use std::borrow::Cow;
use std::error::Error;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use serde_json::Value;
use sqlx::encode::IsNull;
use sqlx::types::Uuid;
use sqlx::{Database, Encode, Sqlite, Type};
use sqlx::sqlite::SqliteArgumentValue;

use crate::utils::typpe_conversion::{unwrap_option, ValueConvert};

/// Enum representing different types of database field values.
#[derive(Default, Debug, Clone, PartialEq)]
pub enum DataKind<'a> {
    /// Text type (string).
    Text(Cow<'a, str>), // SQLite: TEXT

    /// Integer type.
    Integer(i64), // SQLite: INTEGER (includes i8, i16, i32, u8, u16, u32)

    /// Real number type.
    Real(f64), // SQLite: REAL

    /// Date and time types
    DateTime(NaiveDateTime), // SQLite: DATETIME (TEXT, INTEGER, REAL)
    DateTimeUtc(DateTime<Utc>), // SQLite: DATETIME (TEXT, INTEGER, REAL)
    Date(NaiveDate), // SQLite: DATE (TEXT only)
    Time(NaiveTime), // SQLite: TIME (TEXT only)

    /// BLOB type (byte array).
    Blob(Cow<'a, [u8]>), // SQLite: BLOB

    /// Boolean type.
    Bool(bool), // SQLite: BOOLEAN (internally stored as INTEGER)

    /// JSON type (unstructured JSON data).
    Json(Cow<'a, Value>), // SQLite: TEXT (JSON stored as text)

    /// UUID type (stored as BLOB or TEXT).
    Uuid(Uuid), // SQLite: BLOB or TEXT

    /// Null type.
    #[default]
    Null, // SQLite: NULL
}

impl<'a> Encode<'a, Sqlite> for DataKind<'a> {
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

            // Binary types
            DataKind::Blob(arc) => <Vec<u8> as Encode<'_, Sqlite>>::encode(arc.as_ref().to_vec(), buf),

            // Boolean type
            DataKind::Bool(b) => <i64 as Encode<'_, Sqlite>>::encode(*b as i64, buf),

            // JSON type
            DataKind::Json(arc) => <String as Encode<'_, Sqlite>>::encode(serde_json::to_string(arc.as_ref())?, buf),

            // UUID type
            DataKind::Uuid(uuid) => <String as Encode<'_, Sqlite>>::encode(uuid.to_string(), buf),
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
            String => |v: &String| DataKind::Text(Cow::Owned(v.clone())),
            &str => |v: &'a str| DataKind::Text(Cow::Borrowed(v)),
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
            Vec<u8> => |v: &Vec<u8>| DataKind::Blob(Cow::Owned(v.clone())),
            &[u8] => |v: &&'a [u8]| DataKind::Blob(Cow::Borrowed(*v)),
            Value => |v: &Value| DataKind::Json(Cow::Owned(v.clone())),
            Uuid => |v: &Uuid| DataKind::Uuid(*v)
        );
    }
}

// Implement automatic conversion from common types to DataKind
macro_rules! impl_from {
    ($type:ty, $variant:expr) => {
        impl<'a> From<$type> for DataKind<'a> {
            fn from(item: $type) -> Self {
                $variant(item)
            }
        }
    };
}

// Basic types
impl_from!(String, |value: String| DataKind::Text(Cow::Owned(value)));
impl_from!(&'a str, |value: &'a str| DataKind::Text(Cow::Borrowed(value)));
impl_from!(Vec<u8>, |value: Vec<u8>| DataKind::Blob(Cow::Owned(value)));
impl_from!(&'a [u8], |value: &'a [u8]| DataKind::Blob(Cow::Borrowed(value)));

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

// JSON type
impl_from!(Value, |value: Value| DataKind::Json(Cow::Owned(value)));

// UUID type
impl_from!(Uuid, DataKind::Uuid);