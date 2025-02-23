use std::any::Any;
use std::borrow::Cow;
use std::error::Error;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc, NaiveDateTime};
use sqlx::TypeInfo;
use sqlx::{encode::IsNull, Encode, Type};
use sqlx::mysql::{MySql, MySqlTypeInfo};
use serde_json::Value;
use rust_decimal::Decimal;

use crate::common::util::{check_empty, unwrap_option};

/// 数据类型枚举，支持 MySQL 主要类型系统
#[derive(Debug, Clone)]
pub enum DataKind<'a> {
    // 基础类型
    Null,
    Boolean(bool),
    
    // 数值类型
    Int(i64),
    UInt(u64),
    Float(f32),
    Double(f64),
    Decimal(Decimal),
    
    // 字符串类型
    String(Cow<'a, str>),
    
    // 二进制类型
    Blob(Cow<'a, [u8]>),
    
    // 时间类型
    Date(NaiveDate),
    Time(NaiveTime),
    DateTime(NaiveDateTime),
    Timestamp(DateTime<Utc>),
    
    // 特殊类型
    Json(Cow<'a, Value>),
    Enum(Cow<'a, str>),
    
    // 错误处理
    Unsupported(&'static str),
}

impl<'a> Encode<'a, MySql> for DataKind<'a> {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> Result<IsNull, Box<dyn Error + Send + Sync>> {
        match self {
            // 基础类型
            DataKind::Null => Ok(IsNull::Yes),
            DataKind::Boolean(b) => <bool as Encode<'_, MySql>>::encode(*b, buf),
            
            // 数值类型
            DataKind::Int(i) => <i64 as Encode<'_, MySql>>::encode(*i, buf),
            DataKind::UInt(u) => <u64 as Encode<'_, MySql>>::encode(*u, buf),
            DataKind::Float(f) => <f32 as Encode<'_, MySql>>::encode(*f, buf),
            DataKind::Double(d) => <f64 as Encode<'_, MySql>>::encode(*d, buf),
            DataKind::Decimal(d) => <Decimal as Encode<'_, MySql>>::encode(*d, buf),
            
            // 字符串类型
            DataKind::String(s) => <Cow<'_, str> as Encode<'_, MySql>>::encode(Cow::Borrowed(s), buf),
            
            // 二进制类型
            DataKind::Blob(b) => <Vec<u8> as Encode<'_, MySql>>::encode(b.to_vec(), buf),
            
            // 时间类型
            DataKind::Date(d) => <NaiveDate as Encode<'_, MySql>>::encode(*d, buf),
            DataKind::Time(t) => <NaiveTime as Encode<'_, MySql>>::encode(*t, buf),
            DataKind::DateTime(dt) => <NaiveDateTime as Encode<'_, MySql>>::encode(*dt, buf),
            DataKind::Timestamp(ts) => <DateTime<Utc> as Encode<'_, MySql>>::encode(*ts, buf),
            
            // 特殊类型
            DataKind::Json(j) => <Value as Encode<'_, MySql>>::encode(j.clone().into_owned(), buf),
            DataKind::Enum(s) => <Cow<'_, str> as Encode<'_, MySql>>::encode(Cow::Borrowed(s), buf),
            
            // 错误处理
            DataKind::Unsupported(msg) => Err(msg.to_string().into()),
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
            "CHAR" | "VARCHAR" | "TEXT"
            | "INT" | "BIGINT" | "TINYINT"
            | "DATE" | "DATETIME" | "TIMESTAMP"
            | "BLOB" | "MEDIUMBLOB" | "LONGBLOB"
            | "ENUM" | "JSON" | "DECIMAL"
        )
    }
}

impl<'a> DataKind<'a> {
    /// 动态获取准确的类型信息
    pub fn get_type_info(&self) -> MySqlTypeInfo {
        match self {
            DataKind::String(_) => <str as Type<MySql>>::type_info(),
            DataKind::Int(_) => <i64 as Type<MySql>>::type_info(),
            DataKind::UInt(_) => <u64 as Type<MySql>>::type_info(),
            DataKind::Float(_) => <f32 as Type<MySql>>::type_info(),
            DataKind::Double(_) => <f64 as Type<MySql>>::type_info(),
            DataKind::Decimal(_) => <Decimal as Type<MySql>>::type_info(),
            DataKind::Date(_) => <NaiveDate as Type<MySql>>::type_info(),
            DataKind::Time(_) => <NaiveTime as Type<MySql>>::type_info(),
            DataKind::DateTime(_) => <NaiveDateTime as Type<MySql>>::type_info(),
            DataKind::Timestamp(_) => <DateTime<Utc> as Type<MySql>>::type_info(),
            DataKind::Blob(_) => <Vec<u8> as Type<MySql>>::type_info(),
            DataKind::Boolean(_) => <bool as Type<MySql>>::type_info(),
            DataKind::Enum(_) => <str as Type<MySql>>::type_info(),
            DataKind::Json(_) => <Value as Type<MySql>>::type_info(),
            _ => <str as Type<MySql>>::type_info(),
        }
    }
}

/// 将任意类型的值转换为 `DataKind` 枚举类型
pub fn value_convert<'a>(value: &dyn Any) -> DataKind<'a> {
    macro_rules! try_convert {
        ($($type:ty => $variant:expr),*) => {
            $(if let Some(v) = unwrap_option::<$type>(value) {
                return $variant(v);
            })*
            $(if unwrap_option::<$type>(value).is_none() {
                return DataKind::Null;
            })*
        };
    }
    try_convert!(
        String => |v: &String| DataKind::String(Cow::Owned(v.clone())),
        &str => |v: &'a str| DataKind::String(Cow::Borrowed(v)),
        i32 => |v: &i32| DataKind::Int(*v as i64),
        u32 => |v: &u32| DataKind::UInt(*v as u64),
        i64 => |v: &i64| DataKind::Int(*v),
        u64 => |v: &u64| DataKind::UInt(*v),
        f32 => |v: &f32| DataKind::Float(*v),
        f64 => |v: &f64| DataKind::Double(*v),
        Decimal => |v: &Decimal| DataKind::Decimal(*v),
        NaiveDate => |v: &NaiveDate| DataKind::Date(*v),
        NaiveTime => |v: &NaiveTime| DataKind::Time(*v),
        NaiveDateTime => |v: &NaiveDateTime| DataKind::DateTime(*v),
        DateTime<Utc> => |v: &DateTime<Utc>| DataKind::Timestamp(*v),
        Vec<u8> => |v: &Vec<u8>| DataKind::Blob(Cow::Owned(v.clone())),
        &[u8] => |v: &&'a [u8]| DataKind::Blob(Cow::Borrowed(*v)),
        bool => |v: &bool| DataKind::Boolean(*v),
        Value => |v: &Value| DataKind::Json(Cow::Owned(v.clone()))
    );

    DataKind::Unsupported("Unknown type")
}

pub fn is_empty(value: &dyn Any) -> bool {
    check_empty(value, |value| {
        if value.is::<Option<bool>>()
        || value.is::<Option<String>>()
        || value.is::<Option<&str>>()
        || value.is::<Option<Vec<u8>>>()
        || value.is::<Option<&[u8]>>()
        || value.is::<Option<i32>>()
        || value.is::<Option<u32>>()
        || value.is::<Option<u64>>()
        || value.is::<Option<NaiveDate>>()
        || value.is::<Option<NaiveTime>>()
        || value.is::<Option<NaiveDateTime>>()
        || value.is::<Option<i64>>()
        || value.is::<Option<f32>>()
        || value.is::<Option<f64>>()
        || value.is::<Option<Decimal>>()
        || value.is::<Option<DateTime<Utc>>>()
        || value.is::<Option<Value>>() {
            return unwrap_option::<()>(value).is_none();
        }
        if let Some(kind) = unwrap_option::<DataKind>(value) {
            return matches!(kind, DataKind::Null);
        }

        false
    })
}

// 为常见类型实现 From trait
impl<'a> From<String> for DataKind<'a> {
    fn from(item: String) -> Self {
        DataKind::String(Cow::Owned(item))
    }
}

impl<'a> From<&'a str> for DataKind<'a> {
    fn from(item: &'a str) -> Self {
        DataKind::String(Cow::Borrowed(item))
    }
}

impl<'a> From<Vec<u8>> for DataKind<'a> {
    fn from(item: Vec<u8>) -> Self {
        DataKind::Blob(Cow::Owned(item))
    }
}

impl<'a> From<&'a [u8]> for DataKind<'a> {
    fn from(item: &'a [u8]) -> Self {
        DataKind::Blob(Cow::Borrowed(item))
    }
}

impl<'a> From<i32> for DataKind<'a> {
    fn from(item: i32) -> Self {
        DataKind::Int(item as i64)
    }
}

impl<'a> From<u32> for DataKind<'a> {
    fn from(item: u32) -> Self {
        DataKind::UInt(item as u64)
    }
}

impl<'a> From<f32> for DataKind<'a> {
    fn from(item: f32) -> Self {
        DataKind::Float(item)
    }
}

impl<'a> From<f64> for DataKind<'a> {
    fn from(item: f64) -> Self {
        DataKind::Double(item)
    }
}

impl<'a> From<bool> for DataKind<'a> {
    fn from(item: bool) -> Self {
        DataKind::Boolean(item)
    }
}

impl<'a> From<NaiveDate> for DataKind<'a> {
    fn from(item: NaiveDate) -> Self {
        DataKind::Date(item)
    }
}

impl<'a> From<NaiveTime> for DataKind<'a> {
    fn from(item: NaiveTime) -> Self {
        DataKind::Time(item)
    }
}

impl<'a> From<NaiveDateTime> for DataKind<'a> {
    fn from(item: NaiveDateTime) -> Self {
        DataKind::DateTime(item)
    }
}

impl<'a> From<DateTime<Utc>> for DataKind<'a> {
    fn from(item: DateTime<Utc>) -> Self {
        DataKind::Timestamp(item)
    }
}

impl<'a> From<Value> for DataKind<'a> {
    fn from(item: Value) -> Self {
        DataKind::Json(Cow::Owned(item))
    }
}

impl<'a> From<&'a Value> for DataKind<'a> {
    fn from(item: &'a Value) -> Self {
        DataKind::Json(Cow::Borrowed(item))
    }
}

impl<'a> From<Decimal> for DataKind<'a> {
    fn from(item: Decimal) -> Self {
        DataKind::Decimal(item)
    }
}

impl<'a> From<i64> for DataKind<'a> {
    fn from(item: i64) -> Self {
        DataKind::Int(item)
    }
}

impl<'a> From<u64> for DataKind<'a> {
    fn from(item: u64) -> Self {
        DataKind::UInt(item)
    }
}
