use crate::sql::delete::DeleteBuilder;
use crate::sql::filter::Expr;
use crate::sql::filter::ColumnExpr;
use crate::sql::insert::InsertBuilder;
use crate::sql::select::SelectBuilder;
use crate::sql::update::UpdateBuilder;
use super::kind::DataKind;

pub type Select<'a> = SelectBuilder<DataKind<'a>>;
pub type Insert<'a> = InsertBuilder<DataKind<'a>>;
pub type Update<'a> = UpdateBuilder<DataKind<'a>>;
pub type Delete<'a> = DeleteBuilder<DataKind<'a>>;

/// Creates an object to get the field value.
///
/// # Parameters
/// - `name`: Field name.
///
/// # Returns
/// - `Field`: Object to get the field value.
pub fn col<'a>(name: &'a str) -> ColumnExpr<DataKind<'a>> {
    Expr::col(name)
}

impl<'a> Insert<'a> {
    /// Adds an `ON CONFLICT` clause with a `DO UPDATE` action.
    /// NOTE: Supported in Sqlite 3.24+ only.
    pub fn on_conflict_do_update(self, 
        conflict_target: &'a str, 
        excluded_columns: &[&str]
    ) -> Self {
        let mut sql = String::with_capacity(64);
    
        // Append the ON CONFLICT clause
        sql.push_str(" ON CONFLICT (");
        sql.push_str(conflict_target);
        sql.push_str(") DO UPDATE SET ");
    
        // Append the SET clause for excluded columns
        for (i, column) in excluded_columns.iter().enumerate() {
            if i > 0 {
                sql.push_str(", ");
            }
            sql.push_str(column);
            sql.push_str(" = EXCLUDED.");
            sql.push_str(column);
        }
    
        self.append(sql, None)
    }
}
