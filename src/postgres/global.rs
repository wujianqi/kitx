use std::{cell::Cell, sync::OnceLock};

use crate::sql::filter::FilterClause;
use super::kind::DataKind;

static POSTGRES_G_S_D_F: OnceLock<(&'static str, Vec<&'static str>)> = OnceLock::new();

/// Sets the global soft delete field configuration.
///
/// # Parameters
/// - `field_name`: The name of the field used for soft deletes.
/// - `exclude_tables`: A list of table names to exclude from this behavior.
pub fn set_global_soft_delete_field(field_name: &'static str, exclude_tables: Vec<&'static str>) {
    POSTGRES_G_S_D_F.get_or_init(|| (field_name, exclude_tables));
}

/// Retrieves the global soft delete field configuration.
///
/// # Returns
/// - `Option<&'static (String, Vec<String>)>`: If the global soft delete field is set, returns a tuple containing the field name and excluded tables.
/// - `None`: If the global soft delete field has not been configured yet.
pub fn get_global_soft_delete_field() -> Option<&'static (&'static str, Vec<&'static str>)> {
    POSTGRES_G_S_D_F.get()
}

thread_local! {
    static POSTGRES_G_F_S: Cell<Option<(FilterClause<DataKind<'static>>, Vec<&'static str>)>> = Cell::new(None);
}

/// Sets the global filter clause configuration.
///
/// # Parameters
/// - `filter`: A tuple containing the filter clause (`FilterClause<DataKind<'static>>`) and a list of tables to exclude from this filter.
pub fn set_global_filter(filter: FilterClause<DataKind<'static>>, exclude_tables: Vec<&'static str>) {
    POSTGRES_G_F_S.with(|cell| {
        cell.replace(Some((filter, exclude_tables)));
    });
}

/// Retrieves the global filter clause configuration.
///
/// # Returns
/// - `Option<(FilterClause<DataKind<'static>>, Vec<String>)>`: If the global filter clause is set, returns a tuple containing the filter clause and excluded tables.
/// - `None`: If the global filter clause has not been configured yet.
pub fn get_global_filter() -> Option<(FilterClause<DataKind<'static>>, Vec<&'static str>)> {
    POSTGRES_G_F_S.with(|cell| cell.take())
}