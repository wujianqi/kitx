use std::{
    fmt::Debug, sync::Arc
};

use field_access::FieldAccess;
use sqlx::{Database, Error, FromRow};
use crate::{
    builders::table::TableCommon, 
    common::{
        builder::FilterTrait, error::QueryError, operations::OpsBuilderTrait, types::PrimaryKey}, sql::{
         delete::DeleteBuilder, filter::Expr, 
        insert::InsertBuilder, select::SelectBuilder, update::UpdateBuilder,
    }, 
    utils::type_conversion::ValueConvert
};

pub struct SingleKeyTable<'a, T, D, DB, VC>
where    
    T: for<'r> FromRow<'r, DB::Row> + FieldAccess + Default + Clone + Debug + 'a,
    D: Clone + Debug + Default  + Send + Sync,
    DB: Database + 'a,
{
    primary: (&'a str, bool),
    table_common: TableCommon<'a, T, D, DB, VC>,
}

impl<'a, T, D, DB, VC> SingleKeyTable<'a, T, D, DB, VC>
where
    T: for<'r> FromRow<'r, DB::Row> + FieldAccess + Unpin + Send + Sync + Default + Clone + Debug,
    D: Clone + Debug + Default + PartialEq + Send + Sync + From<bool> + From<u64> + 'a,
    DB: Database + 'a,
    VC: ValueConvert<D> + 'a,
{
    pub fn new(
        table_name: &'a str,
        primary: (&'a str, bool),
        soft_delete_config: Option<&'a (&'static str, &'static [&'static str])>,
        global_filters: Option<(Arc<Expr<D>>, Arc<&'static [&'static str]>)>,
    ) -> Self
    {
        let table_common = TableCommon::new(table_name, soft_delete_config, global_filters);

        Self {
            primary,
            table_common,
        }
    }
}

impl<'a, T, D, DB, VC> OpsBuilderTrait<'a, T, D> for SingleKeyTable<'a, T, D, DB, VC>
where
    T: for<'r> FromRow<'r, DB::Row> + FieldAccess + Unpin + Send + Sync + Default + Clone + Debug,
    D: Clone + Debug + Default + Send + Sync + From<u64> + From<bool> + PartialEq + 'a,
    DB: Database + 'a,
    VC: ValueConvert<D> + 'a,
{
    type DeleteBuilder = DeleteBuilder<D>;
    type InsertBuilder = InsertBuilder<D>;
    type SelectBuilder = SelectBuilder<D>;
    type UpdateBuilder = UpdateBuilder<D>;
    
    fn insert_many(&self, entities: Vec<T>) -> Result<InsertBuilder<D>, Error> {
        let primary_name = self.primary.0;
        let auto_inc = self.primary.1;

        self.table_common.insert_many(entities, move |name| {
            name == primary_name && auto_inc
        })
    }

    fn update_one(&self, entity: T) -> Result<UpdateBuilder<D>, Error> {
        self.table_common.update_one(entity, vec![&self.primary.0])
    }

    fn update_by_cond<F>(&self, query_condition: F) -> Result<UpdateBuilder<D>, Error>
        where F: Fn(&mut UpdateBuilder<D>) + Send
    {
        self.table_common.update_by_cond(query_condition)
    }

    fn upsert_many(&self, entities: Vec<T>, use_default_expr: bool) -> Result<(InsertBuilder<D>, Vec<&'a str>, Vec<&'a str>), Error>
    {
        self.table_common.upsert_many(&entities, vec![&self.primary.0], use_default_expr)
    }

    fn delete_by_pk(&self, key: impl Into<PrimaryKey<D>>) -> Result<DeleteBuilder<D>, Error>
    {
        let key = key.into();
        let key_value = match key {
            PrimaryKey::SingleKey(v) => v,
            PrimaryKey::CompositeKey(_) => {
                return Err(QueryError::SingleKeyTypeInvalid.into());                
            }
        };
        self.table_common.delete_by_pk(vec![(self.primary.0, key_value)])
    }

    fn delete_by_cond<F>(&self, query_condition: F) -> Result<Self::DeleteBuilder, Error>
        where F: Fn(&mut Self::DeleteBuilder) + Send
    {
        self.table_common.delete_by_cond(query_condition)
    }

    fn fetch_by_pk(&self, key: impl Into<PrimaryKey<D>>) -> Result<SelectBuilder<D>, Error> {
        let key = key.into();
        let key_value = match key {
            PrimaryKey::SingleKey(v) => v,
            PrimaryKey::CompositeKey(_) => {
                return Err(QueryError::SingleKeyTypeInvalid.into());                
            }
        };

        self.table_common.get_one_by_pk(vec![(self.primary.0, key_value)])
    }

    fn fetch_by_cond<F>(&self, query_condition: F) -> SelectBuilder<D>
        where F: Fn(&mut SelectBuilder<D>)
    {
        self.table_common.fetch_by_cond(query_condition)
    }

    fn get_list_paginated<F>(&self, page_number: u64, page_size: u64, query_condition: F) -> Result<SelectBuilder<D>, Error>
        where F: Fn(&mut SelectBuilder<D>)
    {
        self.table_common.get_list_paginated(page_number, page_size, query_condition)
    }

    fn get_list_by_cursor<F>(&self, limit: u64, query_condition: F) -> Result<SelectBuilder<D>, Error>
        where F: Fn(&mut SelectBuilder<D>)
    {
        self.table_common.get_list_by_cursor(limit, query_condition)
    }

    fn exists<F>(&self, query_condition: F) -> SelectBuilder<D>
        where F: Fn(&mut SelectBuilder<D>)
    {
        self.table_common.exists(query_condition)
    }

    fn count<F>(&self, query_condition: F) -> SelectBuilder<D>
        where F: Fn(&mut SelectBuilder<D>),
        D: Default
    {
        self.table_common.count(query_condition)
    }    
    
    fn is_soft_delete_enabled(&self) -> bool {
        self.table_common.is_soft_delete_enabled()
    }
    
    fn soft_delete_by_pk(&self, key: impl Into<PrimaryKey<D>>) -> Result<Self::UpdateBuilder, Error> {
        let key = key.into();
        let key_value = match key {
            PrimaryKey::SingleKey(v) => v,
            PrimaryKey::CompositeKey(_) => {
                return Err(QueryError::SingleKeyTypeInvalid.into());
            }
        };

        let mut builder = self.table_common.prepare_soft_delete()?;
        builder.and_where_mut(Expr::col(self.primary.0).eq(key_value));
        self.table_common.apply_global_filters(&mut builder);
        Ok(builder)
    }
    
    fn soft_delete_by_cond<F>(&self, query_condition: F) -> Result<Self::UpdateBuilder, Error>
        where F: Fn(&mut Self::UpdateBuilder) + Send
    {
        let mut builder = self.table_common.prepare_soft_delete()?;
        self.table_common.apply_global_filters(&mut builder);
        query_condition(&mut builder);
        Ok(builder)
    }
    
    fn restore_by_pk(&self, key: impl Into<PrimaryKey<D>>) -> Result<UpdateBuilder<D>, Error> {
        let key = key.into();
        let key_value = match key {
            PrimaryKey::SingleKey(v) => v,
            PrimaryKey::CompositeKey(_) => {
                return Err(QueryError::SingleKeyTypeInvalid.into());                
            }
        };

        self.table_common.restore_by_pk(vec![(self.primary.0, key_value)])
    }

    fn restore_by_cond<F>(&self, query_condition: F) -> Result<UpdateBuilder<D>, Error>
        where F: Fn(&mut UpdateBuilder<D>) + Send
    {
        self.table_common.restore_by_cond(query_condition)
    }
    
}