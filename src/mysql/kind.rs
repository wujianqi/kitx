use std::any::Any;
use std::borrow::Cow;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde_json::Value;

/// 数据类型枚举，用于表示不同类型的数据库字段值。
#[derive(Debug, Clone)]
pub enum DataKind<'a> {
    /// 文本类型（字符串），对应 MySQL 的 VARCHAR 或 TEXT。
    Text(Cow<'a, str>),
    /// 空文本类型（Option<String> 为 None）。
    TextNone,
    /// 布尔类型（bool），对应 MySQL 的 TINYINT(1)。
    Boolean(bool),
    /// 空布尔类型（Option<bool> 为 None）。
    BooleanNone,
    /// 整数类型（i32），对应 MySQL 的 INT。
    Integer(i32),
    /// 空整数类型（Option<i32> 为 None）。
    IntegerNone,
    /// 长整数类型（i64），对应 MySQL 的 BIGINT。
    Long(i64),
    /// 空长整数类型（Option<i64> 为 None）。
    LongNone,
    /// 浮点数类型（f32），对应 MySQL 的 FLOAT。
    Float(f32),
    /// 空浮点数类型（Option<f32> 为 None）。
    FloatNone,
    /// 双精度浮点数类型（f64），对应 MySQL 的 DOUBLE。
    Double(f64),
    /// 空双精度浮点数类型（Option<f64> 为 None）。
    DoubleNone,
    /// 日期时间类型（DateTime<Utc>），对应 MySQL 的 DATETIME。
    DateTime(DateTime<Utc>),
    /// 空日期时间类型（Option<DateTime<Utc>> 为 None）。
    DateTimeNone,
    /// UUID 类型，对应 MySQL 的 CHAR(36) 或 VARCHAR(36)。
    Uuid(Uuid),
    /// 空 UUID 类型（Option<Uuid> 为 None）。
    UuidNone,
    /// BLOB类型（字节数组），对应 MySQL 的 BLOB。
    Blob(Cow<'a, [u8]>),
    /// 空BLOB类型（Option<Vec<u8>> 为 None）。
    BlobNone,
    /// JSON 类型，对应 MySQL 的 JSON。
    Json(Cow<'a, Value>),
    /// 空 JSON 类型（Option<Value> 为 None）。
    JsonNone,
    /// 枚举类型，对应 MySQL 的 ENUM。
    Enum(Cow<'a, str>),
    /// 空枚举类型（Option<String> 为 None）。
    EnumNone,
    /// 集合类型，对应 MySQL 的 SET。
    Set(Cow<'a, str>),
    /// 空集合类型（Option<String> 为 None）。
    SetNone,
    /// 不支持的类型。
    Unsupported,
}

/// 辅助函数，递归展开任意层数的 Option 包装。
///
/// # 参数
/// - `value`: 任意类型的引用。
///
/// # 返回
/// - `Option<&T>`: 如果成功解包，则返回内部值的引用；否则返回 None。
fn unwrap_option<T: Any>(value: &dyn Any) -> Option<&T> {
    if value.is::<Option<T>>() {
        let opt = value.downcast_ref::<Option<T>>();
        match opt.and_then(|x| x.as_ref()) {
            Some(inner) => Some(inner),
            None => None,
        }
    } else if value.is::<T>() {
        value.downcast_ref::<T>()
    } else {
        None
    }
}

/// 将任意类型的值转换为 `DataKind` 枚举类型。
///
/// # 参数
/// - `value`: 任意类型的引用。
///
/// # 返回
/// - `DataKind`: 转换后的数据类型。
pub fn value_convert<'a>(value: &dyn Any) -> DataKind<'a> {
    // 根据实际类型进行转换。
    if let Some(s) = unwrap_option::<String>(value) {
        DataKind::Text(Cow::Owned(s.clone()))
    } else if let Some(s) = unwrap_option::<&str>(value) {
        DataKind::Text(Cow::Borrowed(s))
    } else if let Some(b) = unwrap_option::<bool>(value) {
        DataKind::Boolean(*b)
    } else if let Some(i) = unwrap_option::<i32>(value) {
        DataKind::Integer(*i)
    } else if let Some(l) = unwrap_option::<i64>(value) {
        DataKind::Long(*l)
    } else if let Some(f) = unwrap_option::<f32>(value) {
        DataKind::Float(*f)
    } else if let Some(d) = unwrap_option::<f64>(value) {
        DataKind::Double(*d)
    } else if let Some(dt) = unwrap_option::<DateTime<Utc>>(value) {
        DataKind::DateTime(*dt)
    } else if let Some(uuid) = unwrap_option::<Uuid>(value) {
        DataKind::Uuid(*uuid)
    } else if let Some(blob) = unwrap_option::<Vec<u8>>(value) {
        DataKind::Blob(Cow::Owned(blob.clone()))
    } else if let Some(blob) = unwrap_option::<&[u8]>(value) {
        DataKind::Blob(Cow::Borrowed(blob))
    } else if let Some(json) = unwrap_option::<Value>(value) {
        DataKind::Json(Cow::Owned(json.clone()))
    } else {
        // 检查 Option 类型并返回相应的 'None' 变体。
        if value.is::<Option<String>>() {
            DataKind::TextNone
        } else if value.is::<Option<bool>>() {
            DataKind::BooleanNone
        } else if value.is::<Option<i32>>() {
            DataKind::IntegerNone
        } else if value.is::<Option<i64>>() {
            DataKind::LongNone
        } else if value.is::<Option<f32>>() {
            DataKind::FloatNone
        } else if value.is::<Option<f64>>() {
            DataKind::DoubleNone
        } else if value.is::<Option<DateTime<Utc>>>() {
            DataKind::DateTimeNone
        } else if value.is::<Option<Uuid>>() {
            DataKind::UuidNone
        } else if value.is::<Option<Vec<u8>>>() {
            DataKind::BlobNone
        } else if value.is::<Option<Value>>() {
            DataKind::JsonNone
        } else {
            DataKind::Unsupported
        }
    }
}

pub fn is_empty(value: &dyn Any) -> bool {
    // 检查空字符串
    if let Some(s) = unwrap_option::<String>(value) {
        return s.is_empty() || s.to_lowercase() == "null";
    }
    if let Some(s) = unwrap_option::<&str>(value) {
        return s.is_empty() || s.to_lowercase() == "null";
    }
    // 检查空二进制数据
    if let Some(blob) = unwrap_option::<Vec<u8>>(value) {
        return blob.is_empty();
    }
    if let Some(blob) = unwrap_option::<&[u8]>(value) {
        return blob.is_empty();
    }
    // 检查 Option 类型最终包含 None 的情况
    if value.is::<Option<String>>()
        || value.is::<Option<bool>>()
        || value.is::<Option<i32>>()
        || value.is::<Option<i64>>()
        || value.is::<Option<f32>>()
        || value.is::<Option<f64>>()
        || value.is::<Option<DateTime<Utc>>>()
        || value.is::<Option<Uuid>>()
        || value.is::<Option<Vec<u8>>>()
        || value.is::<Option<Value>>() {
        return unwrap_option::<()>(value).is_none();
    }

    false
}

impl<'a> DataKind<'a> {
    /// 将字符串转换为 `Enum` 类型。
    pub fn as_enum(item: impl Into<Cow<'a, str>>) -> Self {
        DataKind::Enum(item.into())
    }

    /// 将字符串转换为 `Set` 类型。
    pub fn as_set(item: impl Into<Cow<'a, str>>) -> Self {
        DataKind::Set(item.into())
    }
}

// 实现 From trait
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

impl From<bool> for DataKind<'_> {
    fn from(item: bool) -> Self {
        DataKind::Boolean(item)
    }
}

impl From<i32> for DataKind<'_> {
    fn from(item: i32) -> Self {
        DataKind::Integer(item)
    }
}

impl From<i64> for DataKind<'_> {
    fn from(item: i64) -> Self {
        DataKind::Long(item)
    }
}

impl From<f32> for DataKind<'_> {
    fn from(item: f32) -> Self {
        DataKind::Float(item)
    }
}

impl From<f64> for DataKind<'_> {
    fn from(item: f64) -> Self {
        DataKind::Double(item)
    }
}

/// 定义宏来简化 DataKind 绑定逻辑。
///
/// # 参数
/// - `$query`: SQL 查询构建器。
/// - `$value`: 要绑定的值。
///
/// # 返回
/// - `Result<_, Error>`: 如果绑定成功，则返回查询构建器；如果遇到不支持的数据类型，则返回错误。
#[macro_export]
macro_rules! mysql_field_bind {
    ($query:expr, $value:expr) => {{
        use crate::mysql::kind::DataKind::*;
        use chrono::{DateTime, Utc};
        use serde_json::Value;
        use sqlx::Error;

        match $value {
            Text(s) => $query.bind(s.into_owned()),
            Boolean(b) => $query.bind(b),
            Integer(n) => $query.bind(n),
            Long(n) => $query.bind(n),
            Float(n) => $query.bind(n),
            Double(n) => $query.bind(n),
            DateTime(dt) => $query.bind(dt),
            Uuid(uuid) => $query.bind(uuid.to_string()),
            Blob(b) => $query.bind(b.into_owned()),
            Json(json) => $query.bind(json.into_owned()),
            Enum(s) => $query.bind(s.into_owned()),
            Set(s) => $query.bind(s.into_owned()),
            TextNone => $query.bind(None::<String>),
            BooleanNone => $query.bind(None::<bool>),
            IntegerNone => $query.bind(None::<i32>),
            LongNone => $query.bind(None::<i64>),
            FloatNone => $query.bind(None::<f32>),
            DoubleNone => $query.bind(None::<f64>),
            DateTimeNone => $query.bind(None::<DateTime<Utc>>),
            UuidNone => $query.bind(None::<String>),
            BlobNone => $query.bind(None::<Vec<u8>>),
            JsonNone => $query.bind(None::<Value>),
            EnumNone => $query.bind(None::<String>),
            SetNone => $query.bind(None::<String>),
            Unsupported => return Err(Error::Decode("Unsupported data type encountered".into())),
        }
    }};
}
