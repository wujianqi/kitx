use std::fmt::Debug;
use std::marker::PhantomData;

use field_access::FieldAccess;
use sqlx::{Database, FromRow};

use crate::{
    common::builder::FilterTrait, 
    sql::filter::Expr, 
    utils::value::ValueConvert
};

pub struct TableQueryBuilder<'a, T, D, DB, VC>
where
    T: for<'r> FromRow<'r, DB::Row> + FieldAccess + Unpin + Send + Sync + Default,
    D: Clone + Debug + Send + Sync + 'a,
    DB: Database,
    VC: ValueConvert<D>,
{
    pub table_name: &'a str,
    pub primary_key: (&'a str, bool),
    pub soft_delete_config: Option<&'a (&'static str, Vec<&'static str>)>, 
    pub global_filters: Option<(Expr<D>, Vec<&'static str>)>,    
    _marker: PhantomData<(T, DB, VC)>,
}


impl<'a, T, D, DB, VC> TableQueryBuilder<'a, T, D, DB, VC>
where
    T: for<'r> FromRow<'r, DB::Row> + FieldAccess + Unpin + Send + Sync + Default,
    D: Clone + Debug + Send + Sync + From<bool> + 'a,
    DB: Database,
    VC: ValueConvert<D>,
{
    pub fn new(
        table_name: &'a str,
        primary_key: (&'a str, bool),
        soft_delete_config: Option<&'a (&'static str, Vec<&'static str>)>, 
        global_filters: Option<(Expr<D>, Vec<&'static str>)>,
    ) -> Self {
        Self {
            table_name,
            primary_key,
            soft_delete_config,
            global_filters,
            _marker: PhantomData,
        }
    }

    pub fn apply_global_filters<'b, W>(&self, builder: &mut W)
    where
        W: FilterTrait<D, Expr = Expr<D>> + 'b,
    {
        if let Some((soft_delete_field, exclude_tables)) = &self.soft_delete_config {
            if !exclude_tables.contains(&self.table_name) {
                builder.where_mut(Expr::col(soft_delete_field).eq(false));
            }
        }

        if let Some((filter, exclude_tables)) = &self.global_filters {
            if !exclude_tables.contains(&self.table_name) {
                builder.where_mut(filter.clone());
            }
        }
    }
}