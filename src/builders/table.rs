use std::marker::PhantomData;
use std::sync::Arc;
use std::fmt::Debug;

use field_access::FieldAccess;
use sqlx::{Database, Error, FromRow};

use crate::common::builder::FilterTrait;
use crate::common::error::{QueryError, SoftDeleteError};
use crate::sql::agg::Func;
use crate::sql::filter::Expr;
use crate::sql::delete::DeleteBuilder;
use crate::sql::insert::InsertBuilder;
use crate::sql::select::SelectBuilder;
use crate::sql::update::UpdateBuilder;
use crate::utils::type_conversion::{is_default_pk, ValueConvert};

pub struct TableCommon<'a, T, D, DB, VC>
where
    T: Default,
    D: Clone + Debug,    
    DB: Database + 'a,
{
    table_name: &'a str,
    soft_delete_config: Option<&'a (&'static str, &'static [&'static str])>,
    global_filters: Option<(Arc<Expr<D>>, Arc<&'static [&'static str]>)>,
    _marker: PhantomData<(T, DB, VC)>,
}

impl<'a, D, T, DB, VC> TableCommon<'a, T, D, DB, VC>
where
    D: Clone + Debug + Send + Sync + From<bool> + Default + PartialEq + From<u64> +'a,
    T: for<'r> FromRow<'r, DB::Row> + FieldAccess + Default + Clone + Debug + 'a,
    DB: Database + 'a,
    VC: ValueConvert<D> + 'a,
{
    pub fn new(
        table_name: &'a str,
        soft_delete_config: Option<&'a (&'static str, &'static [&'static str])>,
        global_filters: Option<(Arc<Expr<D>>, Arc<&'static [&'static str]>)>,
    ) -> Self {
        Self {
            table_name,
            soft_delete_config,
            global_filters,
             _marker: PhantomData,
        }
    }

    pub fn apply_global_filters<W>(&self, builder: &mut W)
    where
        W: FilterTrait<D, Expr = Expr<D>>,
    {
        if let Some((soft_delete_field, exclude_tables)) = self.soft_delete_config {
            if !exclude_tables.contains(&self.table_name) {
                builder.and_where_mut(Expr::col(soft_delete_field).eq(false));
            }
        }

        if let Some((filter, exclude_tables)) = &self.global_filters {
            if !exclude_tables.contains(&self.table_name) {
                builder.and_where_mut(filter.clone());
            }
        }
    }

    pub fn prepare_soft_delete(&self) -> Result<UpdateBuilder<D>, Error> {
        if let Some((column, exclude_tables)) = self.soft_delete_config {
            if !exclude_tables.contains(&self.table_name) {
                let builder = UpdateBuilder::table(self.table_name)
                    .set_cols(&[column], vec![D::from(true)]);
                return Ok(builder);
            }
            return Err(SoftDeleteError::NoTableNameDefined.into());
        }
        Err(SoftDeleteError::SoftDeleteConfigNotSet.into())
    }
    
    pub fn select_builder(&self) -> SelectBuilder<D> {
        SelectBuilder::columns(T::default().field_names()).from(self.table_name)
    }

    pub fn insert_many<F>(&self, entities: Vec<T>, is_primary_key: F) -> Result<InsertBuilder<D>, Error>
    where
        F: Fn(&str) -> bool,
    {
        if entities.is_empty() {
            return Err(QueryError::NoEntitiesProvided.into());
        }

        let mut cols_names = Vec::new();
        let mut all_cols_values = Vec::new();

        for (i, entity) in entities.into_iter().enumerate() {
            let mut cols_values = Vec::new();

            for (name, field) in entity.fields() {
                if is_primary_key(name) {
                    continue;
                }
                if i == 0 {
                    cols_names.push(name);
                }
                let value = VC::convert(field.as_any());
                cols_values.push(value);
            }

            all_cols_values.push(cols_values);
        }

        if cols_names.is_empty() {
            return Err(QueryError::ColumnsListEmpty.into());
        }
        let builder = InsertBuilder::into(self.table_name)
            .columns(&cols_names)
            .values(all_cols_values);
        Ok(builder)
    }

    pub fn update_one(&self, entity: T, pk_cols: Vec<&'a str>) -> Result<UpdateBuilder<D>, Error> {
        let mut cols_names = Vec::new();
        let mut cols_values = Vec::new();
        let mut pks = Vec::new();

        for (name, field) in entity.fields() {
            if pk_cols.contains(&name) {
                let value = VC::convert(field.as_any());
                pks.push((name, value));
            } else {
                let value = VC::convert(field.as_any());
                cols_names.push(name);
                cols_values.push(value);
            }
        }

        if cols_names.is_empty() {
            return Err(QueryError::ColumnsListEmpty.into());
        }

        if pks.is_empty() {
            return Err(QueryError::PrimaryKeyNotFound("No primary key found in the entity".to_string()).into());
        }

        let mut builder = UpdateBuilder::table(self.table_name)
            .set_cols(&cols_names, cols_values);

        for (pk_name, key_value) in pks {
            builder.and_where_mut(Expr::<D>::col(pk_name).eq(key_value));
        }

        Ok(builder)
    }

    pub fn update_by_cond<F>(&self, query_condition: F) -> Result<UpdateBuilder<D>, Error>
    where
        F: Fn(&mut UpdateBuilder<D>) + Send,
    {
        let mut builder = UpdateBuilder::table(self.table_name);
        //self.apply_global_filters(&mut builder);
        query_condition(&mut builder);
        Ok(builder)
    }

    pub fn upsert_many(&self, entities: &[T], pk_cols: Vec<&'a str>, use_default_expr: bool) -> Result<(InsertBuilder<D>, Vec<&'a str>, Vec<&'a str>), Error>
    {
        if entities.is_empty() {
            return Err(QueryError::NoEntitiesProvided.into());
        }

        let mut cols_names = Vec::new();
        let mut values_list = Vec::new();
        let mut default_ids = Vec::new();

        for (u, entity) in entities.into_iter().enumerate() {
            let len = entity.fields().len();
            let mut current_fields = Vec::with_capacity(len);
            let mut current_values = Vec::with_capacity(len);

            for (i, (name, field)) in entity.fields().into_iter().enumerate() {
                let is_pk = pk_cols.contains(&name);
                let is_default = is_default_pk(field.as_any());  
                current_fields.push(name);

                if is_pk && is_default {
                    current_values.push(D::default());
                    if use_default_expr {
                        default_ids.push(u * len + i);
                    }
                } else {
                    current_values.push(VC::convert(field.as_any()));
                }
            }
            
            if cols_names.is_empty() {
                cols_names.extend(current_fields);
            }
            values_list.push(current_values);
        }

        if cols_names.is_empty() {
            return Err(QueryError::ColumnsListEmpty.into());
        }

        let mut builder = InsertBuilder::into(self.table_name)
            .columns(&cols_names)
            .values(values_list);

        if use_default_expr {
            for item in default_ids {
                builder.replace_expr_at_mut(item, "DEFAULT");
            }
        }

        Ok((builder, cols_names, pk_cols))
    }

    pub fn delete_by_pk(&self, keys: Vec<(&'a str, D)>) -> Result<DeleteBuilder<D>, Error>
    {
        if keys.is_empty() {
            return Err(Error::from(QueryError::NoPrimaryKeyDefined));
        }

        let mut builder = DeleteBuilder::from(self.table_name);
        for (col_name, value) in keys {
            builder.and_where_mut(Expr::col(col_name).eq(value));
        }
        self.apply_global_filters(&mut builder);
        
        Ok(builder)
    }

    pub fn delete_by_cond<F>(&self, query_condition: F) -> Result<DeleteBuilder<D>, Error>
    where
        F: Fn(&mut DeleteBuilder<D>) + Send,
    {
        let mut builder = DeleteBuilder::from(self.table_name);
        self.apply_global_filters(&mut builder);
        query_condition(&mut builder);
        Ok(builder)
    }

    pub fn fetch_by_cond<F>(&self, query_condition: F) -> SelectBuilder<D>
        where F: Fn(&mut SelectBuilder<D>),
    {
        let mut builder = self.select_builder();
        self.apply_global_filters(&mut builder);
        query_condition(&mut builder);
        builder
    }

    pub fn get_one_by_pk(&self, keys: Vec<(&'a str, D)>) -> Result<SelectBuilder<D>, Error> {
        if keys.is_empty() {
            return Err(Error::from(QueryError::NoPrimaryKeyDefined));
        }

        let mut builder = self.select_builder();
        for (col_name, value) in keys {
            builder.and_where_mut(Expr::col(col_name).eq(value));
        }
        self.apply_global_filters(&mut builder);
        
        Ok(builder)
    }    

    pub fn get_list_paginated<F>(&self, page_number: u64, page_size: u64, query_condition: F) -> Result<SelectBuilder<D>, Error>
        where F: Fn(&mut SelectBuilder<D>),
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

    pub fn get_list_by_cursor<F>(&self, limit: u64, query_condition: F) -> Result<SelectBuilder<D>, Error>
        where F: Fn(&mut SelectBuilder<D>),
    {
        if limit < 1 {
            return Err(QueryError::LimitInvalid.into());
        }

        let mut builder = self.select_builder()
            .limit_offset(D::from(limit), None::<D>);
        
        self.apply_global_filters(&mut builder);
        query_condition(&mut builder);
        
        Ok(builder)
    }

    pub fn exists<F>(&self, query_condition: F) -> SelectBuilder<D>
        where F: Fn(&mut SelectBuilder<D>)
    {
        let mut builder = SelectBuilder::columns(&["1"]).from(self.table_name);
        self.apply_global_filters(&mut builder);
        query_condition(&mut builder);
        builder
    }

    pub fn count<F>(&self, query_condition: F) -> SelectBuilder<D>
        where F: Fn(&mut SelectBuilder<D>),
        D: Default
    {
        let agg = Func::default().count("*", "");
        let mut builder = SelectBuilder::empty_columns()
            .aggregate(agg)
            .from(self.table_name);
        
        self.apply_global_filters(&mut builder);
        query_condition(&mut builder);
        builder
    }

    pub fn restore_by_pk(&self, keys: Vec<(&'a str, D)>) -> Result<UpdateBuilder<D>, Error> {
        if !self.is_soft_delete_enabled() {
            return Err(SoftDeleteError::RestoreOperationNotSupported.into());
        }
        
        if let Some((column, exclude_tables)) = &self.soft_delete_config {
            if !exclude_tables.contains(&self.table_name) {
                if keys.is_empty() {
                    return Err(SoftDeleteError::SoftDeleteColumnTypeInvalid.into());
                }

                let mut builder = UpdateBuilder::table(self.table_name)
                    .set_cols(&[column], vec![D::from(false)]);

                for (col_name, value) in keys {
                    builder.and_where_mut(Expr::col(col_name).eq(value));
                }
                
                return Ok(builder);
            }
        }
        
        Err(SoftDeleteError::RestoreOperationNotSupported.into())
    }

    pub fn restore_by_cond<F>(&self, query_condition: F) -> Result<UpdateBuilder<D>, Error>
    where
        F: Fn(&mut UpdateBuilder<D>) + Send,
    {
        if !self.is_soft_delete_enabled() {
            return Err(SoftDeleteError::RestoreOperationNotSupported.into());
        }

        let mut builder = self.prepare_soft_delete()?;
        query_condition(&mut builder);
        Ok(builder)
    }

    pub fn is_soft_delete_enabled(&self) -> bool {
        self.soft_delete_config.is_some()
    }
}