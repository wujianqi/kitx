use crate::common::builder::{BuilderCondition, BuilderTrait};
use crate::sql::{builder::Builder, filter::Field};
use super::kind::DataKind;

/// PostgreSQL-specific SQL builder.
pub type QueryBuilder<'a> = Builder<DataKind<'a>>;
/// PostgreSQL-specific SQL condition builder.
pub type QueryCondition<'a> = BuilderCondition<'a, QueryBuilder<'a>>;

/// Creates an object for retrieving field values.
///
/// # Parameters
/// - `name`: Field name.
///
/// # Returns
/// - `Field`: Object for retrieving field values.
pub fn field<'a>(name: &'a str) -> Field<'a, DataKind<'a>> {
    Field::get(name)
}

// PostgreSQL-specific methods
impl<'a> QueryBuilder<'a> {
    /// Adds an ON CONFLICT clause.
    pub fn on_conflict(
        &mut self,
        table: &str,
        columns: &[&str],
        values: Vec<Vec<DataKind<'a>>>,
        update_columns: &[&str],
    ) -> &mut Self {
        // Reuse the logic from insert_into to generate the base SQL and parameters
        let mut builder = Builder::insert_into(table, columns, values.clone());

        // Create the ON CONFLICT clause
        let update_clause = update_columns
            .iter()
            .map(|col| format!("{} = ?", col))
            .collect::<Vec<String>>()
            .join(", ");
        let sqlstr = format!(" ON CONFLICT ({}) DO UPDATE SET {}", columns.join(", "), update_clause);

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
            vec!["John".into(), "30".into()]
        ];
        let builder = QueryBuilder::insert_into("users", &["name", "age"], values);
        assert_eq!(builder.build().0, "INSERT INTO users ( name, age ) VALUES (?, ?)");
    }
}