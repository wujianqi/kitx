use std::{
    fmt::Debug, sync::Arc
};

use field_access::FieldAccess;
use sqlx::{Database, Error, FromRow};
use crate::{
    builders::table::TableCommon, 
    common::{
        builder::FilterTrait, error::{QueryError, SoftDeleteError}, operations::OpsBuilderTrait, types::PrimaryKey}, 
        sql::{
            delete::DeleteBuilder, filter::Expr, insert::InsertBuilder, select::SelectBuilder, update::UpdateBuilder
        }, 
    utils::type_conversion::ValueConvert
};


pub struct CompositeKeyTable<'a, T, D, DB, VC>
where    
    T: for<'r> FromRow<'r, DB::Row> + FieldAccess + Default + Clone + Debug + 'a,
    D: Clone + Debug + Default  + Send + Sync,
    DB: Database + 'a,
{
    primarys: Vec<&'a str>,
    table_common: TableCommon<'a, T, D, DB, VC>,
}

impl<'a, T, D, DB, VC> CompositeKeyTable<'a, T, D, DB, VC>
where
    T: for<'r> FromRow<'r, DB::Row> + FieldAccess + Unpin + Send + Sync + Default + Clone + Debug,
    D: Clone + Debug + Default + PartialEq + Send + Sync + From<bool> + From<u64> + 'a,
    DB: Database + 'a,
    VC: ValueConvert<D> + 'a,
{
    pub fn new(
        table_name: &'a str,
        primarys: Vec<&'a str>,
        soft_delete_config: Option<&'a (&'static str, &'static [&'static str])>,
        global_filters: Option<(Arc<Expr<D>>, Arc<&'static [&'static str]>)>,
    ) -> Self
    {
        let table_common = TableCommon::new(table_name, soft_delete_config, global_filters);

        Self {
            primarys,
            table_common,
        }
    }
}

impl<'a, T, D, DB, VC> OpsBuilderTrait<'a, T, D> for CompositeKeyTable<'a, T, D, DB, VC>
where
    T: for<'r> FromRow<'r, DB::Row> + FieldAccess + Unpin + Send + Sync + Default + Clone + Debug,
    D: Clone + Debug + Send + Sync + From<u64> + From<bool> + Default + PartialEq + 'a,
    DB: Database + 'a,
    VC: ValueConvert<D> + 'a,
{

    type DeleteBuilder = DeleteBuilder<D>;
    type InsertBuilder = InsertBuilder<D>;
    type SelectBuilder = SelectBuilder<D>;
    type UpdateBuilder = UpdateBuilder<D>;

    fn insert_many(&self, entities: Vec<T>) -> Result<Self::InsertBuilder, Error> {
        self.table_common.insert_many(entities, |_| false)
    }

    fn upsert_many(&self, entities: Vec<T>, use_default_expr: bool) -> Result<(InsertBuilder<D>, Vec<&'a str>, Vec<&'a str>), Error> 
    {
        self.table_common.upsert_many(&entities, self.primarys.clone(), use_default_expr)
    }

    fn update_one(&self, entity: T) -> Result<Self::UpdateBuilder, Error> {
        self.table_common.update_one(entity, self.primarys.clone())
    }

    fn update_by_cond<F>(&self, query_condition: F) -> Result<Self::UpdateBuilder, Error>
    where
        F: Fn(&mut Self::UpdateBuilder) + Send,
    {
        self.table_common.update_by_cond(query_condition)
    }

    fn delete_by_pk(&self, key: impl Into<PrimaryKey<D>>) -> Result<Self::DeleteBuilder, Error> {
        let key = key.into();
        let composite_key = match key {
            PrimaryKey::CompositeKey(keys) => keys,
            PrimaryKey::SingleKey(_) => return Err(QueryError::CompositeKeyTypeInvalid.into()),
        };

        if composite_key.len() != self.primarys.len() {
            return Err(QueryError::NoPrimaryKeyDefined.into());
        }

        self.table_common.delete_by_pk(
            self.primarys.iter()
                .zip(composite_key)
                .map(|(k, v)| (*k, v))
                .collect()
        )
    }

    fn delete_by_cond<F>(&self, query_condition: F) -> Result<Self::DeleteBuilder, Error>
    where
        F: Fn(&mut Self::DeleteBuilder) + Send,
    {
        self.table_common.delete_by_cond(query_condition)
    }


    fn fetch_by_pk(&self, key: impl Into<PrimaryKey<D>>) -> Result<Self::SelectBuilder, Error> {
        let key = key.into();
        let composite_key = match key {
            PrimaryKey::CompositeKey(keys) => keys,
            PrimaryKey::SingleKey(_) => return Err(QueryError::CompositeKeyTypeInvalid.into()),
        };

        if composite_key.len() != self.primarys.len() {
            return Err(QueryError::NoPrimaryKeyDefined.into());
        }

        self.table_common.get_one_by_pk(
            self.primarys.iter()
                .zip(composite_key)
                .map(|(k, v)| (*k, v))
                .collect()
        )
    }

    fn fetch_by_cond<F>(&self, query_condition: F) -> Self::SelectBuilder
    where
        F: Fn(&mut Self::SelectBuilder),
    {
        self.table_common.fetch_by_cond(query_condition)
    }

    fn get_list_paginated<F>(&self, page_number: u64, page_size: u64, query_condition: F) -> Result<Self::SelectBuilder, Error>
    where
        F: Fn(&mut Self::SelectBuilder),
    {
        self.table_common.get_list_paginated(page_number, page_size, query_condition)
    }

    fn get_list_by_cursor<F>(&self, limit: u64, query_condition: F) -> Result<Self::SelectBuilder, Error>
    where
        F: Fn(&mut Self::SelectBuilder),
    {
        self.table_common.get_list_by_cursor(limit, query_condition)
    }

    fn exists<F>(&self, query_condition: F) -> Self::SelectBuilder
    where
        F: Fn(&mut Self::SelectBuilder),
    {
        self.table_common.exists(query_condition)
    }

    fn count<F>(&self, query_condition: F) -> Self::SelectBuilder
    where
        F: Fn(&mut Self::SelectBuilder),
        D: Default,
    {
        self.table_common.count(query_condition)
    }


    fn is_soft_delete_enabled(&self) -> bool {
        self.table_common.is_soft_delete_enabled()
    }

    fn soft_delete_by_pk(&self, key: impl Into<PrimaryKey<D>>) -> Result<Self::UpdateBuilder, Error> {
        let key = key.into();
        let composite_key = match key {
            PrimaryKey::CompositeKey(keys) => keys,
            PrimaryKey::SingleKey(_) => return Err(QueryError::CompositeKeyTypeInvalid.into()),
        };

        if composite_key.len() != self.primarys.len() {
            return Err(QueryError::NoPrimaryKeyDefined.into());
        }

        let mut builder = self.table_common.prepare_soft_delete()?;
        for (col_name, value) in self.primarys.iter().zip(composite_key) {
            builder.and_where_mut(Expr::col(col_name).eq(value));
        }
        self.table_common.apply_global_filters(&mut builder);
        Ok(builder)
    }

    fn soft_delete_by_cond<F>(&self, query_condition: F) -> Result<Self::UpdateBuilder, Error>
    where
        F: Fn(&mut Self::UpdateBuilder) + Send,
    {
        if !self.is_soft_delete_enabled() {
            return Err(SoftDeleteError::SoftDeleteConfigNotSet.into());
        }

        let mut builder = self.table_common.prepare_soft_delete()?;
        self.table_common.apply_global_filters(&mut builder);
        query_condition(&mut builder);
        Ok(builder)
    }

    fn restore_by_pk(&self, key: impl Into<PrimaryKey<D>>) -> Result<Self::UpdateBuilder, Error> {
        let key = key.into();
        let composite_key = match key {
            PrimaryKey::CompositeKey(keys) => keys,
            PrimaryKey::SingleKey(_) => return Err(QueryError::CompositeKeyTypeInvalid.into()),
        };

        if composite_key.len() != self.primarys.len() {
            return Err(QueryError::NoPrimaryKeyDefined.into());
        }

        self.table_common.restore_by_pk(
            self.primarys.iter()
                .zip(composite_key)
                .map(|(k, v)| (*k, v))
                .collect()
        )
    }
    
    fn restore_by_cond<F>(&self, query_condition: F) -> Result<Self::UpdateBuilder, Error>
    where
        F: Fn(&mut Self::UpdateBuilder) + Send,
    {
        self.table_common.restore_by_cond(query_condition)
    }

}
