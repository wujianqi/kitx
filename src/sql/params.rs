use std::fmt::Debug;
use std::borrow::Cow;
use chrono::{DateTime, Utc};

/// Defines an enumeration compatible with multiple types
#[derive(Clone, Debug, Default)]
pub enum Value<'a> {
    Int(i32),
    Float(f32),
    Text(Cow<'a, str>),
    Bool(bool),
    Timestamp(DateTime<Utc>),
    Blob(Cow<'a, [u8]>),
    #[default]
    Null,
}

/// Provides Into type conversion methods
impl<'a> From<i32> for Value<'a> {
    fn from(value: i32) -> Self {
        Value::Int(value)
    }
}

impl<'a> From<f32> for Value<'a> {
    fn from(value: f32) -> Self {
        Value::Float(value)
    }
}

impl<'a> From<String> for Value<'a> {
    fn from(value: String) -> Self {
        Value::Text(Cow::Owned(value))
    }
}

impl<'a> From<&'a str> for Value<'a> {
    fn from(value: &'a str) -> Self {
        Value::Text(Cow::Borrowed(value))
    }
}

impl<'a> From<bool> for Value<'a> {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl<'a> From<DateTime<Utc>> for Value<'a> {
    fn from(value: DateTime<Utc>) -> Self {
        Value::Timestamp(value)
    }
}

impl<'a> From<Vec<u8>> for Value<'a> {
    fn from(value: Vec<u8>) -> Self {
        Value::Blob(Cow::Owned(value))
    }
}

impl<'a> From<&'a [u8]> for Value<'a> {
    fn from(value: &'a [u8]) -> Self {
        Value::Blob(Cow::Borrowed(value))
    }
}

impl<'a, T> From<Option<T>> for Value<'a>
where
    T: Into<Value<'a>>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(inner) => inner.into(),
            None => Value::Null,
        }
    }
}