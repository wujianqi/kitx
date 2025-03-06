use crate::common::builder::{BuilderCondition, BuilderTrait};
use crate::sql::{builder::Builder, filter::FieldValue};
use super::kind::DataKind;

/// MySQL-specific SQL builder.
pub type QueryBuilder<'a> = Builder<DataKind<'a>>;
/// MySQL-specific SQL condition builder.
pub type QueryCondition<'a> = BuilderCondition<'a, QueryBuilder<'a>>;

/// Creates an object for retrieving field values.
///
/// # Parameters
/// - `name`: Field name.
///
/// # Returns
/// - `FieldValue`: Object for retrieving field values.
pub fn field<'a>(name: &'a str) -> FieldValue<'a, DataKind<'a>> {
    FieldValue::get(name)
}

// MySQL-specific methods
impl<'a> QueryBuilder<'a> {
    /// Adds an ON DUPLICATE KEY UPDATE clause.
    pub fn on_duplicate_key_update(
        &mut self,
        table: &str,
        columns: &[&str],
        values: Vec<Vec<DataKind<'a>>>,
        update_columns: &[&str],
    ) -> &mut Self { // Return &mut Self
        // Reuse the logic from insert_into to generate the base SQL and parameters
        let mut builder = Builder::insert_into(table, columns, values.clone());

        // Create the ON DUPLICATE KEY UPDATE clause
        let update_clause = update_columns
            .iter()
            .map(|col| format!("{} = ?", col))
            .collect::<Vec<String>>()
            .join(", ");
        let sqlstr = format!(" ON DUPLICATE KEY UPDATE {}", update_clause);

        // Add the values to be updated to cols_values
        let mut vals = Vec::new();
        for row in &values {
            for col in update_columns {
                if let Some(index) = columns.iter().position(|&c| c == *col) {
                    vals.push(row[index].clone());
                }
            }
        }

        // Add the SQL and parameters to the builder
        builder.append(&sqlstr, Some(vals));

        // Return a mutable reference to the current builder
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let values = vec![
            vec![DataKind::String("John".into()), DataKind::String("30".into())]
        ];
        let builder = QueryBuilder::insert_into("users", &["name", "age"], values);
        assert_eq!(builder.build().0, "INSERT INTO users ( name, age ) VALUES (?, ?)");
    }
}