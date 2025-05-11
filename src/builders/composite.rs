use std::{fmt::Debug, marker::PhantomData, sync::Arc};
use field_access::FieldAccess;
use sqlx::{Database, Error, FromRow};
use crate::{
    common::{builder::FilterTrait, error::OperationError},
    sql::{
        agg::Func, delete::DeleteBuilder, filter::Expr, insert::InsertBuilder, select::SelectBuilder, update::UpdateBuilder
    },
    utils::typpe_conversion::ValueConvert,
};

pub struct CompositeKeyTable<'a, T, D, DB, VC>
where
    D: Clone + Debug + Send + Sync + 'a,
{
    table_name: &'a str,
    primarys: Vec<&'a str>,
    soft_delete_config: Option<&'a (&'static str, &'static [&'static str])>,
    global_filters: Option<(Arc<Expr<D>>, Arc<&'static [&'static str]>)>,
    _marker: PhantomData<(T, DB, VC)>,
}

impl<'a, T, D, DB, VC> CompositeKeyTable<'a, T, D, DB, VC>
where
    T: for<'r> FromRow<'r, DB::Row> + FieldAccess + Unpin + Send + Sync + Default,
    D: Clone + Debug + Send + Sync + From<u64> + From<bool> + 'a,
    DB: Database,
    VC: ValueConvert<D>,
{
    pub fn new(
        table_name: &'a str,
        primarys: Vec<&'a str>,
        soft_delete_config: Option<&'a (&'static str, &'static [&'static str])>,
        global_filters: Option<(Arc<Expr<D>>, Arc<&'static [&'static str]>)>,
    ) -> Self {
        Self {
            table_name,
            primarys,
            soft_delete_config,
            global_filters,
            _marker: PhantomData,
        }
    }
    pub fn is_soft_delete_enabled(&self) -> bool {
        if let Some((_, exclude_tables)) = &self.soft_delete_config {
            !exclude_tables.contains(&self.table_name)
        } else {
            false
        }
    }    

    pub fn delete_by_keys(&self, keys: &[(&str, D)]) -> Result<DeleteBuilder<D>, Error> {
        let mut builder = DeleteBuilder::from(self.table_name);
        self.apply_primary_keys(&mut builder, keys)?;
        self.apply_global_filters(&mut builder);
        Ok(builder)
    }

    pub fn delete_by_cond<F>(&self, query_condition: F) -> DeleteBuilder<D>
    where
        F: Fn(&mut DeleteBuilder<D>) + Send + 'a,
    {
        let mut builder = DeleteBuilder::from(self.table_name);
        self.apply_global_filters(&mut builder);
        query_condition(&mut builder);
        builder
    }
    
    pub fn soft_delete_by_cond<F>(&self, query_condition: F) -> Result<UpdateBuilder<D>, Error>
    where
        F: Fn(&mut DeleteBuilder<D>) + Send + 'a,
    {
        let mut delete_builder = DeleteBuilder::from(self.table_name);
        query_condition(&mut delete_builder);
        
        let mut builder = self.prepare_soft_delete()?;
        for condition in delete_builder.take_where_clauses() {
            builder.and_where_mut(condition);
        }
        Ok(builder)
    }

    pub fn soft_delete_by_keys(&self, keys: &[(&str, D)]) -> Result<UpdateBuilder<D>, Error> {
        let mut builder = self.prepare_soft_delete()?;
        self.apply_primary_keys(&mut builder, keys)?;
        self.apply_global_filters(&mut builder);
        Ok(builder)
    }

    pub fn restore_one(&self, key: impl Into<D> + Send) -> Result<UpdateBuilder<D>, Error> {
        if let Some((column, exclude_tables)) = &self.soft_delete_config {
            if !exclude_tables.contains(&self.table_name) {
                return Ok(UpdateBuilder::table(self.table_name)
                    .set_cols(&[column], vec![D::from(false)])
                    .and_where(Expr::col(self.primarys[0]).eq(key)));
            }
        }
        Err(OperationError::RestoreOperationNotSupported.into())
    }

    pub fn restore_by_keys(&self, keys: &[(&str, D)]) -> Result<UpdateBuilder<D>, Error> {
        if let Some((column, exclude_tables)) = self.soft_delete_config {
            if !exclude_tables.contains(&self.table_name) {
                let mut builder = UpdateBuilder::table(self.table_name)
                    .set_cols(&[column], vec![D::from(false)]);
                self.apply_primary_keys(&mut builder, keys)?;
                self.apply_global_filters(&mut builder);
                return Ok(builder);
            }
            return Err(OperationError::NoTableNameDefined.into());
        }
        Err(OperationError::RestoreOperationNotSupported.into())
    }

    // Implement query operations
    pub fn get_one_by_keys(&self, keys: &[(&str, D)]) -> Result<SelectBuilder<D>, Error> {
        let mut builder = self.select_builder();
        self.apply_primary_keys(&mut builder, keys)?;
        self.apply_global_filters(&mut builder);
        Ok(builder)
    }

    pub fn get_list<F>(&self, query_condition: F) -> SelectBuilder<D>
    where
        F: Fn(&mut SelectBuilder<D>) + 'a,
    {
        let mut builder = self.select_builder();
        self.apply_global_filters(&mut builder);
        query_condition(&mut builder);
        builder
    }

    // Paginated query with offset-based pagination
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
            return Err(OperationError::PageNumberInvalid.into());
        }

        let offset = (page_number - 1) * page_size;
        let mut builder = self.select_builder()
            .limit_offset(D::from(page_size), Some(D::from(offset)));

        self.apply_global_filters(&mut builder);
        query_condition(&mut builder);

        Ok(builder)
    }

    // Cursor-based pagination implementation
    pub fn get_list_by_cursor<F>(
        &self,
        limit: u64,
        query_condition: F,
    ) -> Result<SelectBuilder<D>, Error>
    where
        F: Fn(&mut SelectBuilder<D>) + 'a,
    {
        if limit == 0 {
            return Err(OperationError::LimitInvalid.into());
        }

        let mut builder = self.select_builder()
            .limit_offset(D::from(limit), None::<D>);

        self.apply_global_filters(&mut builder);
        query_condition(&mut builder);

        Ok(builder)
    }

    // Data insertion operations
    pub fn insert_one(&self, entity: T) -> Result<InsertBuilder<D>, Error> 
    where 
        D: Default + PartialEq,
    {
        let mut cols_names = Vec::new();
        let mut cols_values = Vec::new();
    
        for (name, field) in entity.fields() {
            if self.primarys.contains(&name) {
                let value = VC::convert(field.as_any());
                if value == D::default() {
                    return Err(OperationError::PrimaryKeyNotFound(name.to_string()).into());
                }
                cols_names.push(name);
                cols_values.push(value);
            } else {
                cols_names.push(name);
                cols_values.push(VC::convert(field.as_any()));
            }
        }
    
        if cols_names.is_empty() {
            return Err(OperationError::ColumnsListEmpty.into());
        }
    
        Ok(InsertBuilder::into(self.table_name)
            .columns(&cols_names)
            .values(vec![cols_values]))
    }
    
    pub fn insert_many(&self, entities: Vec<T>) -> Result<InsertBuilder<D>, OperationError>
    where 
        D: Default + PartialEq,
    {
        if entities.is_empty() {
            return Err(OperationError::NoEntitiesProvided);
        }
    
        let mut cols_names = Vec::new();
        let mut all_cols_values = Vec::new();
    
        for (i, entity) in entities.into_iter().enumerate() {
            let mut cols_values = Vec::new();
            
            for (name, field) in entity.fields() {
                if i == 0 {
                    cols_names.push(name);
                }
                let value = VC::convert(field.as_any());

                if self.primarys.contains(&name) && value == D::default() {
                    return Err(OperationError::PrimaryKeyNotFound(name.to_string()));
                }
                
                cols_values.push(value);
            }
            all_cols_values.push(cols_values);
        }
    
        Ok(InsertBuilder::into(self.table_name)
            .columns(&cols_names)
            .values(all_cols_values))
    }

    pub fn count<F>(&self, query_condition: F) -> SelectBuilder<D>
    where
        F: Fn(&mut SelectBuilder<D>) + 'a,
        D: Default,
    {
        let agg = Func::default().count("*", "");
        let mut builder = SelectBuilder::columns(&[])
            .aggregate(agg)
            .from(self.table_name);

        self.apply_global_filters(&mut builder);
        query_condition(&mut builder);
        builder
    }

    // Helper methods
    fn select_builder(&self) -> SelectBuilder<D> {
        let column_names = T::default().fields()
            .map(|(name, _)| name)
            .collect::<Vec<_>>();
        
        SelectBuilder::columns(&column_names).from(self.table_name)
    }

    fn apply_primary_keys<W>(&self, builder: &mut W, keys: &[(&str, D)]) -> Result<(), Error>
    where
        W: FilterTrait<D, Expr = Expr<D>>,
    {
        for (key_name, key_value) in keys {
            if !self.primarys.contains(key_name) {
                return Err(OperationError::PrimaryKeyNotFound(key_name.to_string()).into());
            }
            builder.and_where_mut(Expr::col(*key_name).eq(key_value.clone()));
        }
        Ok(())
    }

    fn apply_global_filters<'b, W>(&self, builder: &mut W)
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
                builder.and_where_mut(filter.clone());
            }
        }
    }

    fn prepare_soft_delete(&self) -> Result<UpdateBuilder<D>, Error> {
        if let Some((column, exclude_tables)) = self.soft_delete_config {
            if let Some(field) = T::default().field(column) {
                if field.as_bool().is_none() {
                    return Err(OperationError::SoftDeleteColumnTypeInvalid.into());
                }
            }
    
            if !exclude_tables.contains(&self.table_name) {
                let builder = UpdateBuilder::table(self.table_name)
                    .set_cols(&[column], vec![D::from(true)]);
                return Ok(builder);
            }
            return Err(OperationError::NoTableNameDefined.into());
        }
        Err(OperationError::SoftDeleteConfigNotSet.into())
    }

}