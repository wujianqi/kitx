use crate::common::builder::BuilderCondition;
use crate::sql::{builder::Builder, filter::FieldValue};
use super::kind::DataKind;

/// SQLite-specific SQL builder.
pub type QueryBuilder<'a> = Builder<DataKind<'a>>;

/// SQLite-specific SQL query condition.
pub type QueryCondition<'a> = BuilderCondition<'a, QueryBuilder<'a>>;

/// Creates an object to get the field value.
///
/// # Parameters
/// - `name`: Field name.
///
/// # Returns
/// - `FieldValue`: Object to get the field value.
pub fn field<'a>(name: &'a str) -> FieldValue<'a, DataKind<'a>> {
    FieldValue::get(name)
}
