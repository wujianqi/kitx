use std::{
    fmt::Debug, marker::PhantomData, sync::Arc
};

use field_access::FieldAccess;
use sqlx::{Database, FromRow};
use crate::{
    common::builder::FilterTrait,
    sql::filter::Expr,
    utils::type_conversion::ValueConvert,
};

pub struct SingleKeyTable<'a, T, D, DB, VC>
where
    D: Clone + Debug + Send + Sync + 'a,
{
    pub(super) table_name: &'a str,
    pub(super) primary: (&'a str, bool),
    pub(super) soft_delete_config: Option<&'a (&'static str, &'static [&'static str])>,
    pub(super) global_filters: Option<(Arc<Expr<D>>, Arc<&'static [&'static str]>)>,
    _marker: PhantomData<(T, DB, VC)>,
}

impl<'a, T, D, DB, VC> SingleKeyTable<'a, T, D, DB, VC>
where
    T: for<'r> FromRow<'r, DB::Row> + FieldAccess + Unpin + Send + Sync + Default,
    D: Clone + Debug + Send + Sync + From<bool> + 'a,
    DB: Database,
    VC: ValueConvert<D>,
{
    pub fn new(
        table_name: &'a str,
        primary: (&'a str, bool),
        soft_delete_config: Option<&'a (&'static str, &'static [&'static str])>,
        global_filters: Option<(Arc<Expr<D>>, Arc<&'static [&'static str]>)>,
    ) -> Self {
        Self {
            table_name,
            primary,
            soft_delete_config,
            global_filters,
            _marker: PhantomData,
        }
    }

    pub(super) fn apply_global_filters<'b, W>(&self, builder: &mut W)
    where
        W: FilterTrait<D, Expr = Expr<D>> + 'b,
    {
        if let Some((soft_delete_field, exclude_tables)) = self.soft_delete_config {
            if !exclude_tables.contains(&self.table_name) {
                builder.and_where_mut(Expr::col(soft_delete_field).eq(false));
            }
        }

        if let Some((filter, exclude_tables)) = &self.global_filters {
            if !exclude_tables.contains(&self.table_name) {
                builder.and_where_mut(filter.as_ref().clone());
            }
        }
    }
}