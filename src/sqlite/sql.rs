use crate::sql::delete::DeleteBuilder;
use crate::sql::filter::Expr;
use crate::sql::filter::ColumnExpr;
use crate::sql::insert::InsertBuilder;
use crate::sql::select::SelectBuilder;
use crate::sql::update::UpdateBuilder;
use crate::utils::value::ValueConvert;
use super::kind::DataKind;

pub type Convert<'a> = dyn ValueConvert<DataKind<'a>>;
pub type DataExpr<'a> = Expr<DataKind<'a>>;
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

