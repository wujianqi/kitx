use crate::sql::{
    delete::DeleteBuilder, 
    filter::{ColumnExpr, Expr}, 
    insert::InsertBuilder, 
    select::SelectBuilder, 
    update::UpdateBuilder
};

use super::kind::DataKind;

pub type Select<'a> = SelectBuilder<DataKind<'a>>;
pub type Insert<'a> = InsertBuilder<DataKind<'a>>;
pub type Update<'a> = UpdateBuilder<DataKind<'a>>;
pub type Delete<'a> = DeleteBuilder<DataKind<'a>>;

/// Creates an object for retrieving field values.
///
/// # Parameters
/// - `name`: Field name.
///
/// # Returns
/// - `Field`: Object for retrieving field values.
pub fn col<'a>(name: &'a str) -> ColumnExpr<DataKind<'a>> {
    Expr::col(name)
}

// MySQL-specific methods
impl<'a> Insert<'a> {
    /// Adds an ON DUPLICATE KEY UPDATE clause.
    pub fn on_duplicate_key_update(
        self,
        update_columns: &[&str],
    ) -> Self {
        let mut sql = String::with_capacity(64);
        sql.push_str(" ON DUPLICATE KEY UPDATE ");

        for (i, column) in update_columns.iter().enumerate() {
            if i > 0 {
                sql.push_str(", ");
            }
            sql.push_str(column);
            sql.push_str(" = VALUES(");
            sql.push_str(column);
            sql.push_str(")");
        }
    
        self.append(sql, None)
    }
}

