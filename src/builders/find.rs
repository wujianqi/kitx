use std::fmt::Debug;
use field_access::FieldAccess;
use sqlx::{Database, Error, FromRow};

use crate::common::error::QueryError;
use crate::sql::agg::Func;
use crate::utils::typpe_conversion::ValueConvert;
use crate::sql::{
    filter::Expr,
    select::SelectBuilder,
};

use super::single::SingleKeyTable;

impl<'a, T, D, DB, VC> SingleKeyTable<'a, T, D, DB, VC>
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

    pub fn get_one_by_key(&self, id: impl Into<D> + Send) -> Result<SelectBuilder<D>, Error> 
    where 
        D: Default + PartialEq,
    {
        let key = id.into();
        if key == D::default() {
            return Err(QueryError::NoPrimaryKeyDefined.into());
        }

        let mut builder = self.select_builder()
            .and_where(Expr::col(self.primary.0).eq(key));
        self.apply_global_filters(&mut builder);
        Ok(builder)
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

    pub fn get_list_paginated<F>(
        &self,
        page_number: u64,
        page_size: u64,
        query_condition: F,
    ) -> Result<SelectBuilder<D>, Error>
    where
        F: Fn(&mut SelectBuilder<D>) + 'a,
    {
        if page_number == 0 || page_size == 0 {
            return Err(QueryError::PageNumberInvalid.into());
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
    ) -> Result<SelectBuilder<D>, Error>
    where
        F: Fn(&mut SelectBuilder<D>) + 'a,
    {
        if limit == 0 {
            return Err(QueryError::LimitInvalid.into());
        }

        let mut builder = self.select_builder()
            .limit_offset(D::from(limit), None::<D>);

        self.apply_global_filters(&mut builder);
        query_condition(&mut builder);

        Ok(builder)
    }

    pub fn exists<F>(&self, query_condition: F) -> SelectBuilder<D>
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
        D: Default,
    {
        let agg = Func::default().count(&self.primary.0, "");
        let mut builder = SelectBuilder::columns(&[])
            .aggregate(agg)
            .from(self.table_name);

        self.apply_global_filters(&mut builder);
        query_condition(&mut builder);
        builder
    }
    
}