use std::fmt::Debug;
use field_access::FieldAccess;
use sqlx::{Database, FromRow};

use crate::common::error::OperationError;
use crate::utils::value::ValueConvert;
use crate::sql::{
    filter::Expr,
    select::SelectBuilder,
};

use super::base::TableQueryBuilder;

impl<'a, T, D, DB, VC> TableQueryBuilder<'a, T, D, DB, VC>
where
    T: for<'r> FromRow<'r, DB::Row> + FieldAccess + Unpin + Send + Sync + Default,
    D: Clone + Debug + Send + Sync + From<u64>  + From<bool> + 'a,
    DB: Database,
    VC: ValueConvert<D>,
{
    fn select_builder(&self) -> SelectBuilder<D> {
        let column_names = T::default().fields()
            .map(|(name, _)| name)
            .collect::<Vec<_>>();

        SelectBuilder::columns(&column_names).from(self.table_name)
    }

    // Query operations
    pub fn get_list<F>(&self, query_condition: F) -> SelectBuilder<D>
    where
        F: Fn(&mut SelectBuilder<D>) + 'a,
    {
        let mut builder = self.select_builder();
        self.apply_global_filters(&mut builder);
        query_condition(&mut builder);
        builder
    }

    pub fn get_by_key(&self, id: impl Into<D> + Send) -> SelectBuilder<D> {
        let id = id.into();
        let mut builder = self.select_builder()
            .where_(Expr::col(self.primary_key.0).eq(id));
        self.apply_global_filters(&mut builder);
        builder
    }

    pub fn get_one<F>(&self, query_condition: F) -> SelectBuilder<D>
    where
        F: Fn(&mut SelectBuilder<D>) + 'a,
    {
        let mut builder = self.select_builder();
        self.apply_global_filters(&mut builder);
        query_condition(&mut builder);
        builder
    }

    pub fn get_list_paginated<F>(
        &self,
        page_number: u64,
        page_size: u64,
        query_condition: F,
    ) -> Result<SelectBuilder<D>, OperationError>
    where
        F: Fn(&mut SelectBuilder<D>) + 'a,
    {
        if page_number == 0 || page_size == 0 {
            return Err(OperationError::new(
                "Page number and page size must be greater than 0".to_string(),
            ));
        }

        let offset = (page_number - 1) * page_size;
        let mut builder = self.select_builder()
            .limit_offset(D::from(page_size), Some(D::from(offset)));

        self.apply_global_filters(&mut builder);
        query_condition(&mut builder);

        Ok(builder)
    }

    pub fn get_list_by_cursor<F>(
        &self,
        limit: u64,
        query_condition: F,
    ) -> Result<SelectBuilder<D>, OperationError>
    where
        F: Fn(&mut SelectBuilder<D>) + 'a,
    {
        if limit == 0 {
            return Err(OperationError::new("Limit must be greater than 0".to_string()));
        }

        let mut builder = self.select_builder()
            .limit_offset(D::from(limit), None::<D>);

        self.apply_global_filters(&mut builder);
        query_condition(&mut builder);

        Ok(builder)
    }

    pub fn exist<F>(&self, query_condition: F) -> SelectBuilder<D>
    where
        F: Fn(&mut SelectBuilder<D>) + 'a,
    {
        let mut builder = SelectBuilder::columns(&["1"]).from(self.table_name);
        self.apply_global_filters(&mut builder);
        query_condition(&mut builder);
        builder
    }

    pub fn count<F>(&self, query_condition: F) -> SelectBuilder<D>
    where
        F: Fn(&mut SelectBuilder<D>) + 'a,
    {
        let mut column_name = String::with_capacity(20);
        column_name.push_str("COUNT(");
        column_name.push_str(self.primary_key.0);
        column_name.push_str(")");

        let mut builder = SelectBuilder::columns(&[&column_name]).from(self.table_name);
        self.apply_global_filters(&mut builder);
        query_condition(&mut builder);
        builder
    }
    
}