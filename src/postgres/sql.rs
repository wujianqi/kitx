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


