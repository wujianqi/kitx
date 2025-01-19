use std::any::Any;
<<<<<<< HEAD
use std::borrow::Cow;
=======
>>>>>>> b0e16200fd5be220e3fbdfdf161f92205821e722
use chrono::{DateTime, Utc};

/// 数据类型枚举，用于表示不同类型的数据库字段值。
#[derive(Debug, Clone)]
<<<<<<< HEAD
pub enum DataKind<'a> {
    /// 文本类型（字符串）。
    Text(Cow<'a, str>),
=======
pub enum DataKind {
    /// 文本类型（字符串）。
    Text(String),
>>>>>>> b0e16200fd5be220e3fbdfdf161f92205821e722
    /// 空文本类型（Option<String> 为 None）。
    TextNone,
    /// 整数类型（i64）。
    Integer(i64),
    /// 空整数类型（Option<i64> 为 None）。
    IntegerNone,
    /// 浮点数类型（f64）。
    Real(f64),
    /// 空浮点数类型（Option<f64> 为 None）。
    RealNone,
    /// 日期时间类型（DateTime<Utc>）。
    DateTime(DateTime<Utc>),
    /// 空日期时间类型（Option<DateTime<Utc>> 为 None）。
    DateTimeNone,
    /// BLOB类型（字节数组）。
<<<<<<< HEAD
    Blob(Cow<'a, [u8]>),
=======
    Blob(Vec<u8>),
>>>>>>> b0e16200fd5be220e3fbdfdf161f92205821e722
    /// 空BLOB类型（Option<Vec<u8>> 为 None）。
    BlobNone,
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
<<<<<<< HEAD
pub fn value_convert<'a>(value: &dyn Any) -> DataKind<'a> {
    // 根据实际类型进行转换。
    if let Some(s) = unwrap_option::<String>(value) {
        DataKind::Text(Cow::Owned(s.clone()))
    } else if let Some(s) = unwrap_option::<&str>(value) {
        DataKind::Text(Cow::Borrowed(s))
=======
pub fn value_convert(value: &dyn Any) -> DataKind {
    // 根据实际类型进行转换。
    if let Some(s) = unwrap_option::<String>(value) {
        DataKind::Text(s.clone())
>>>>>>> b0e16200fd5be220e3fbdfdf161f92205821e722
    } else if let Some(i) = unwrap_option::<i64>(value) {
        DataKind::Integer(*i)
    } else if let Some(b) = unwrap_option::<bool>(value) {
        DataKind::Integer(*b as i64)
    } else if let Some(r) = unwrap_option::<f64>(value) {
        DataKind::Real(*r)
    } else if let Some(dt) = unwrap_option::<DateTime<Utc>>(value) {
        DataKind::DateTime(*dt)
    } else if let Some(blob) = unwrap_option::<Vec<u8>>(value) {
<<<<<<< HEAD
        DataKind::Blob(Cow::Owned(blob.clone()))
    } else if let Some(blob) = unwrap_option::<&[u8]>(value) {
        DataKind::Blob(Cow::Borrowed(blob))
=======
        DataKind::Blob(blob.clone())
>>>>>>> b0e16200fd5be220e3fbdfdf161f92205821e722
    } else {
        // 检查 Option 类型并返回相应的 'None' 变体。
        if value.is::<Option<String>>() {
            DataKind::TextNone
        } else if value.is::<Option<i64>>() {
            DataKind::IntegerNone
        } else if value.is::<Option<f64>>() {
            DataKind::RealNone
        } else if value.is::<Option<DateTime<Utc>>>() {
            DataKind::DateTimeNone
        } else if value.is::<Option<Vec<u8>>>() {
            DataKind::BlobNone
        } else {
            DataKind::Unsupported
        }
    }
}

/// 辅助函数，判断一个值是否为空。
///
/// # 参数
/// - `value`: 任意类型的引用。
///
/// # 返回
/// - `bool`: 如果值为空，则返回 true；否则返回 false。
pub fn is_empty(value: &dyn Any) -> bool {
    // 检查空字符串
    if let Some(s) = unwrap_option::<String>(value) {
        return s.is_empty() || s.to_lowercase() == "null";
    }
<<<<<<< HEAD
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
=======
    /* // 检查零
    if let Some(i) = unwrap_option::<i64>(value) {
        return *i == 0;
    }
    if let Some(r) = unwrap_option::<f64>(value) {
        return *r == 0.0;
    } */
>>>>>>> b0e16200fd5be220e3fbdfdf161f92205821e722
    // 检查 Option 类型最终包含 None 的情况
    if value.is::<Option<String>>()
        || value.is::<Option<i64>>()
        || value.is::<Option<f64>>()
<<<<<<< HEAD
        || value.is::<Option<DateTime<Utc>>>()
        || value.is::<Option<Vec<u8>>>() {
=======
        || value.is::<Option<DateTime<Utc>>>() {
>>>>>>> b0e16200fd5be220e3fbdfdf161f92205821e722
        return unwrap_option::<()>(value).is_none();
    }

    false
}

// 实现从常见类型到 DataKind 的自动转换
<<<<<<< HEAD
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

impl From<i32> for DataKind<'_> {
    fn from(item: i32) -> Self {
        DataKind::Integer(item as i64)
    }
}

impl From<i64> for DataKind<'_> {
=======
impl From<i64> for DataKind {
>>>>>>> b0e16200fd5be220e3fbdfdf161f92205821e722
    fn from(item: i64) -> Self {
        DataKind::Integer(item)
    }
}

<<<<<<< HEAD
impl From<bool> for DataKind<'_> {
=======
impl From<bool> for DataKind {
>>>>>>> b0e16200fd5be220e3fbdfdf161f92205821e722
    fn from(item: bool) -> Self {
        DataKind::Integer(item as i64)
    }
}

<<<<<<< HEAD
impl From<f32> for DataKind<'_> {
    fn from(item: f32) -> Self {
        DataKind::Real(item as f64)
    }
}

impl From<f64> for DataKind<'_> {
=======
impl From<String> for DataKind {
    fn from(item: String) -> Self {
        DataKind::Text(item)
    }
}

impl From<&str> for DataKind {
    fn from(item: &str) -> Self {
        DataKind::Text(item.to_string())
    }
}

impl From<f64> for DataKind {
>>>>>>> b0e16200fd5be220e3fbdfdf161f92205821e722
    fn from(item: f64) -> Self {
        DataKind::Real(item)
    }
}

<<<<<<< HEAD
impl From<DateTime<Utc>> for DataKind<'_> {
=======
impl From<DateTime<Utc>> for DataKind {
>>>>>>> b0e16200fd5be220e3fbdfdf161f92205821e722
    fn from(item: DateTime<Utc>) -> Self {
        DataKind::DateTime(item)
    }
}

<<<<<<< HEAD
impl<'a> From<Cow<'a, [u8]>> for DataKind<'a> {
    fn from(item: Cow<'a, [u8]>) -> Self {
        DataKind::Blob(item)
    }
}

impl From<Vec<u8>> for DataKind<'_> {
    fn from(item: Vec<u8>) -> Self {
        DataKind::Blob(Cow::Owned(item))
    }
}

impl<'a> From<&'a [u8]> for DataKind<'a> {
    fn from(item: &'a [u8]) -> Self {
        DataKind::Blob(Cow::Borrowed(item))
    }
}

=======
>>>>>>> b0e16200fd5be220e3fbdfdf161f92205821e722
/// 定义宏来简化 DataKind 绑定逻辑。
///
/// # 参数
/// - `$query`: SQL 查询构建器。
/// - `$value`: 要绑定的值。
///
/// # 返回
/// - `Result<_, Error>`: 如果绑定成功，则返回查询构建器；如果遇到不支持的数据类型，则返回错误。
#[macro_export]
<<<<<<< HEAD
macro_rules! sqlite_field_bind {
=======
macro_rules! bind_field_value {
>>>>>>> b0e16200fd5be220e3fbdfdf161f92205821e722
    ($query:expr, $value:expr) => {{
        use crate::sqlite::kind::DataKind::*;
        use chrono::{DateTime, Utc};
        use sqlx::Error;

        match $value {
<<<<<<< HEAD
            Text(s) => $query.bind(s.into_owned()),
            Integer(n) => $query.bind(n),
            Real(n) => $query.bind(n),
            DateTime(dt) => $query.bind(dt),
            Blob(b) => $query.bind(b.into_owned()),
=======
            Text(s) => $query.bind(s),
            Integer(n) => $query.bind(n),
            Real(n) => $query.bind(n),
            DateTime(dt) => $query.bind(dt),
            Blob(b) => $query.bind(b),
>>>>>>> b0e16200fd5be220e3fbdfdf161f92205821e722
            TextNone => $query.bind(None::<String>),
            IntegerNone => $query.bind(None::<i64>),
            RealNone => $query.bind(None::<f64>),
            DateTimeNone => $query.bind(None::<DateTime<Utc>>),
            BlobNone => $query.bind(None::<Vec<u8>>),
            Unsupported => return Err(Error::Decode("Unsupported data type encountered".into())),
        }
    }};
<<<<<<< HEAD
}
=======
}
>>>>>>> b0e16200fd5be220e3fbdfdf161f92205821e722
