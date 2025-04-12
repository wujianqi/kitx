use std::fmt::Debug;
use field_access::FieldAccess;
use sqlx::{Database, FromRow};

use crate::{
    common::error::OperationError,
    sql::{delete::DeleteBuilder, filter::Expr, update::UpdateBuilder}, 
    utils::value::ValueConvert
};

use super::base::TableQueryBuilder;


impl<'a, T, D, DB, VC> TableQueryBuilder<'a, T, D, DB, VC>
where
    T: for<'r> FromRow<'r, DB::Row> + FieldAccess + Unpin + Send + Sync + Default,
    D: Clone + Debug + Send + Sync + From<bool> + 'a,
    DB: Database,
    VC: ValueConvert<D>,
{

    // Delete operations
    pub fn delete_by_key(&self, key: impl Into<D> + Send) -> DeleteBuilder<D> {
        let key = key.into();
        DeleteBuilder::from(self.table_name)
            .where_(Expr::col(self.primary_key.0).eq(key))
    }

    pub fn delete_many(&self, keys: Vec<impl Into<D>>) -> Result<DeleteBuilder<D>, OperationError> {
        if keys.is_empty() {
            return Err(OperationError::new("Keys list cannot be empty".to_string()));
        }
        
        let mut builder = DeleteBuilder::from(self.table_name)
            .where_(Expr::col(self.primary_key.0).in_(keys.into_iter().map(|k| k.into()).collect::<Vec<D>>()));
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
    pub fn restore_one(&self, key: impl Into<D> + Send) -> Result<UpdateBuilder<D>, OperationError> {
        let key = key.into();
        if let Some((column, exclude_tables)) = &self.soft_delete_config {
            if !exclude_tables.contains(&self.table_name) {
                return Ok(UpdateBuilder::table(self.table_name)
                    .set_cols(&[column], vec![D::from(false)])
                    .where_(Expr::col(self.primary_key.0).eq(key)));
            }
        }
        Err(OperationError::new("Restore operation not supported without soft delete configuration".to_string()))
    }

    pub fn restore_many(&self, keys: Vec<impl Into<D> + Send>) -> Result<UpdateBuilder<D>, OperationError> {
        let keys: Vec<D> = keys.into_iter().map(|k| k.into()).collect();
        if let Some((column, exclude_tables)) = &self.soft_delete_config {
            if !exclude_tables.contains(&self.table_name) {
                return Ok(UpdateBuilder::table(self.table_name)
                    .set_cols(&[column], vec![D::from(false)])
                    .where_(Expr::col(self.primary_key.0).in_(keys)));
            }
        }
        Err(OperationError::new("Restore operation not supported without soft delete configuration".to_string()))
    }

}
