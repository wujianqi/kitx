use std::fmt::Debug;
use field_access::FieldAccess;
use sqlx::{Database, Error, FromRow};

use crate::{
    common::{builder::FilterTrait, error::OperationError},
    sql::{delete::DeleteBuilder, filter::Expr, update::UpdateBuilder}, 
    utils::typpe_conversion::ValueConvert
};

use super::single::SingleKeyTable;


impl<'a, T, D, DB, VC> SingleKeyTable<'a, T, D, DB, VC>
where
    T: for<'r> FromRow<'r, DB::Row> + FieldAccess + Unpin + Send + Sync + Default,
    D: Clone + Debug + Send + Sync + From<bool> + 'a,
    DB: Database,
    VC: ValueConvert<D>,
{

    pub fn is_soft_delete_enabled(&self) -> bool {
        if let Some((_, exclude_tables)) = &self.soft_delete_config {
            !exclude_tables.contains(&self.table_name)
        } else {
            false
        }
    }

    fn prepare_soft_delete(&self) -> Result<(String, UpdateBuilder<D>), Error> {
        if let Some((column, exclude_tables)) = &self.soft_delete_config {
            if let Some(field) = T::default().field(column) {
                if field.as_bool().is_none() {
                    return Err(OperationError::SoftDeleteColumnTypeInvalid.into());
                }
            }

            if !exclude_tables.contains(&self.table_name) {
                let builder = UpdateBuilder::table(self.table_name)
                    .set_cols(&[column], vec![D::from(true)]);
                return Ok((self.primary.0.to_string(), builder));
            }
            return Err(OperationError::NoTableNameDefined.into());
        }
        Err(OperationError::SoftDeleteConfigNotSet.into())
    }

    pub fn soft_delete_by_key(&self, key: impl Into<D> + Send) -> Result<UpdateBuilder<D>, Error> {
        let (pk_name, builder) = self.prepare_soft_delete()?;
        let updated_builder = builder.and_where(Expr::col(&pk_name).eq(key));
        Ok(updated_builder)
    }

    pub fn soft_delete_many(&self, keys: Vec<impl Into<D> + Send>) -> Result<UpdateBuilder<D>, Error> {
        let keys: Vec<D> = keys.into_iter().map(|k| k.into()).collect();
    
        if keys.is_empty() {
            return Err(OperationError::KeysListEmpty.into());
        }
    
        let (pk_name, builder) = self.prepare_soft_delete()?;
        let updated_builder = builder.and_where(Expr::col(&pk_name).is_in(keys));
        Ok(updated_builder)
    }

    pub fn soft_delete_by_cond<F>(&self, query_condition: F) -> Result<UpdateBuilder<D>, Error>
    where
        F: Fn(&mut DeleteBuilder<D>) + Send + 'a,
    {
        let mut delete_builder = DeleteBuilder::from(self.table_name);
        query_condition(&mut delete_builder);
        
        let (_, mut builder) = self.prepare_soft_delete()?;
        for condition in delete_builder.take_where_clauses() {
            builder.and_where_mut(condition);
        }
        Ok(builder)
    }

    // Delete operations
    pub fn delete_by_key(&self, key: impl Into<D> + Send) -> Result<DeleteBuilder<D>, Error> 
    where 
        D: Default + PartialEq,
    {
        let key = key.into();
        if key == D::default() {
            return Err(OperationError::NoPrimaryKeyDefined.into());
        }
        
        let mut builder = DeleteBuilder::from(self.table_name)
            .and_where(Expr::col(self.primary.0).eq(key));
        self.apply_global_filters(&mut builder);
        Ok(builder)
    }

    pub fn delete_many(&self, keys: Vec<impl Into<D>>) -> Result<DeleteBuilder<D>, Error> {
        if keys.is_empty() {
            return Err(OperationError::KeysListEmpty.into());
        }

        let keys = keys.into_iter().map(|k| k.into()).collect::<Vec<D>>();
        let mut builder = DeleteBuilder::from(self.table_name)
            .and_where(Expr::col(self.primary.0).is_in(keys));
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

    // Restore operations
    pub fn restore_one(&self, key: impl Into<D> + Send) -> Result<UpdateBuilder<D>, Error> {
        if let Some((column, exclude_tables)) = &self.soft_delete_config {
            if !exclude_tables.contains(&self.table_name) {
                return Ok(UpdateBuilder::table(self.table_name)
                    .set_cols(&[column], vec![D::from(false)])
                    .and_where(Expr::col(self.primary.0).eq(key)));
            }
        }

        Err(OperationError::RestoreOperationNotSupported.into())
    }

    pub fn restore_many(&self, keys: Vec<impl Into<D> + Send>) -> Result<UpdateBuilder<D>, Error> {
        let keys: Vec<D> = keys.into_iter().map(|k| k.into()).collect();

        if let Some((column, exclude_tables)) = &self.soft_delete_config {
            if !exclude_tables.contains(&self.table_name) {
                return Ok(UpdateBuilder::table(self.table_name)
                    .set_cols(&[column], vec![D::from(false)])
                    .and_where(Expr::col(self.primary.0).is_in(keys)));
            }
        }

        Err(OperationError::RestoreOperationNotSupported.into())
    }

}
