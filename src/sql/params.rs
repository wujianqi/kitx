use std::fmt::Debug;
use std::borrow::Cow;
use std::time::SystemTime;

/// Defines an enumeration compatible with multiple types
#[derive(Clone, Debug, Default)]
pub enum Value<'a> {
    Int(i32),
    Float(f32),
    Text(Cow<'a, str>),
    Bool(bool),
    Timestamp(SystemTime),
    Blob(Cow<'a, [u8]>),
    #[default]
    Null,
}

macro_rules! impl_from {
    ($type:ty, $variant:expr) => {
        impl<'a> From<$type> for Value<'a> {
            fn from(item: $type) -> Self {
                $variant(item)
            }
        }
    };
}

impl_from!(String, |value: String| Value::Text(Cow::Owned(value)));
impl_from!(&'a str, |value: &'a str| Value::Text(Cow::Borrowed(value)));
impl_from!(Vec<u8>, |value: Vec<u8>| Value::Blob(Cow::Owned(value)));
impl_from!(&'a [u8], |value: &'a [u8]| Value::Blob(Cow::Borrowed(value)));
impl_from!(u32, |value: u32| Value::Int(value as i32));
impl_from!(i32, Value::Int);
impl_from!(f32, Value::Float);
impl_from!(bool, Value::Bool);
impl_from!(SystemTime, Value::Timestamp);


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