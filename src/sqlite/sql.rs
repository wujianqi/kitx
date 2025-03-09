use crate::common::builder::BuilderCondition;
use crate::sql::{builder::Builder, filter::Field};
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
/// - `Field`: Object to get the field value.
pub fn field<'a>(name: &'a str) -> Field<'a, DataKind<'a>> {
    Field::get(name)
}
