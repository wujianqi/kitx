use std::{
    cell::OnceCell, sync::{Arc, OnceLock}
};

use crate::sql::filter::Expr;
use super::kind::DataKind;

static MYSQL_G_S_D_F: OnceLock<(&'static str, &'static [&'static str])> = OnceLock::new();

/// Sets the global soft delete field configuration.
///
/// # Parameters
/// - `field_name`: The name of the field used for soft deletes. The field data type must be boolean.
/// - `exclude_tables`: A list of table names to exclude from this behavior.
pub fn set_global_soft_delete_field(field_name: &'static str, exclude_tables: &'static [&'static str]) {
    MYSQL_G_S_D_F.get_or_init(|| (field_name, exclude_tables));
}

/// Retrieves the global soft delete field configuration.
///
/// # Returns
/// - `Option<&'static (&'static str, HashSet<&'static str>)>`: If the global soft delete field is set, returns a tuple containing the field name and excluded tables.
/// - `None`: If the global soft delete field has not been configured yet.
pub fn get_global_soft_delete_field() -> Option<&'static (&'static str, &'static [&'static str])> {
    MYSQL_G_S_D_F.get()
}

thread_local! {
    static MYSQL_G_F_S: OnceCell<Option<(
        Arc<Expr<DataKind<'static>>>,
        Arc<&'static [&'static str]>
    )>> = OnceCell::new();
}

/// Sets the global filter clause configuration.
///
/// # Parameters
/// - `filter`: A tuple containing the filter clause (`FilterClause<DataKind<'static>>`) and a list of tables to exclude from this filter.
/// Sets the global filter clause configuration.
///
/// # Parameters
/// - `filter`: A tuple containing the filter clause (`FilterClause<DataKind<'static>>`) and a list of tables to exclude from this filter.
pub fn set_global_filter(filter: Expr<DataKind<'static>>, exclude_tables: &'static [&'static str]) {
    let arc_filter = Arc::new(filter);
    let arc_exclude = Arc::new(exclude_tables);

    MYSQL_G_F_S.with(|cell| {
        let _ = cell.set(Some((arc_filter, arc_exclude))).ok();
    });
}

/// Retrieves the global filter clause configuration.
///
/// # Returns
/// - `Option<(FilterClause<DataKind<'static>>, Vec<String>)>`: If the global filter clause is set, returns a tuple containing the filter clause and excluded tables.
/// - `None`: If the global filter clause has not been configured yet.
pub fn get_global_filter() -> Option<(
    Arc<Expr<DataKind<'static>>>,
    Arc<&'static [&'static str]>
)> {
    MYSQL_G_F_S.with(|cell| {
        cell.get().and_then(|opt| {
            if let Some((expr, cols)) = opt {
                Some((expr.clone(), cols.clone()))
            } else {
                None
            }
        })
    })
}