use std::any::Any;
use std::borrow::Cow;
use std::error::Error;
use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::encode::IsNull;
use sqlx::{Database, Encode, Sqlite, Type};
use sqlx::sqlite::SqliteArgumentValue;

use crate::utils::value::{unwrap_option, ValueConvert};

/// Enum representing different types of database field values.
#[derive(Default, Debug, Clone)]
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
    #[default]
    Null,
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
            }
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
            i32 => |v: &i32| DataKind::Integer(*v as i64),
            i64 => |v: &i64| DataKind::Integer(*v),
            f32 => |v: &f32| DataKind::Real(*v as f64),
            f64 => |v: &f64| DataKind::Real(*v),
            bool => |v: &bool| DataKind::Integer(*v as i64),
            NaiveDateTime => |v: &NaiveDateTime| DataKind::DateTime(DateTime::from_naive_utc_and_offset(*v, Utc)),
            DateTime<Utc> => |v: &DateTime<Utc>| DataKind::DateTime(*v),
            Vec<u8> => |v: &Vec<u8>| DataKind::Blob(Cow::Owned(v.clone())),
            &[u8] => |v: &'a [u8]| DataKind::Blob(Cow::Borrowed(v))       
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

impl_from!(String, |value: String| DataKind::Text(Cow::Owned(value)));
impl_from!(&'a str, |value: &'a str| DataKind::Text(Cow::Borrowed(value)));
impl_from!(Vec<u8>, |value: Vec<u8>| DataKind::Blob(Cow::Owned(value)));
impl_from!(&'a [u8], |value: &'a [u8]| DataKind::Blob(Cow::Borrowed(value)));
impl_from!(i32, |value: i32| DataKind::Integer(value as i64));
impl_from!(i64, DataKind::Integer);
impl_from!(u64, |value: u64| DataKind::Integer(value as i64));
impl_from!(f32, |value: f32| DataKind::Real(value as f64));
impl_from!(f64, DataKind::Real);
impl_from!(bool, |value: bool| DataKind::Integer(value as i64));
impl_from!(DateTime<Utc>, DataKind::DateTime);
impl_from!(NaiveDateTime, |value: NaiveDateTime| DataKind::DateTime(DateTime::from_naive_utc_and_offset(value, Utc)));
