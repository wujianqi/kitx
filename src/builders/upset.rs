use std::fmt::Debug;

use field_access::FieldAccess;
use sqlx::{Database, FromRow};

use crate::{
    common::error::OperationError, 
    sql::{filter::Expr, insert::InsertBuilder, update::UpdateBuilder}, 
    utils::value::{is_empty_or_none, ValueConvert}
};

use super::base::TableQueryBuilder;

impl<'a, T, D, DB, VC> TableQueryBuilder<'a, T, D, DB, VC>
where
    T: for<'r> FromRow<'r, DB::Row> + FieldAccess + Unpin + Send + Sync + Default,
    D: Clone + Debug + Send + Sync + 'a,
    DB: Database,
    VC: ValueConvert<D>,
{
    // Insert operations
    pub fn insert_one(&self, entity: T) -> Result<InsertBuilder<D>, OperationError> {
        let mut cols_names = Vec::new();
        let mut cols_values = Vec::new();

        for (name, field) in entity.fields() {
            if name != self.primary_key.0 || !self.primary_key.1 {
                if is_empty_or_none(field.as_any()) {
                    return Err(OperationError::new(format!("Field {} has an invalid value", name)));
                }
                cols_names.push(name);
                let value = VC::convert(field.as_any());
                cols_values.push(value);
            }
        }

        if cols_names.is_empty() {
            return Err(OperationError::new("No valid fields provided for insertion".to_string()));
        }

        Ok(InsertBuilder::into(self.table_name)
            .columns(&cols_names)
            .values(vec![cols_values]))
    }

    pub fn insert_many(&self, entities: Vec<T>) -> Result<InsertBuilder<D>, OperationError> {
        if entities.is_empty() {
            return Err(OperationError::new("No entities provided for insert operation".to_string()));
        }

        let mut cols_names = Vec::new();
        let mut all_cols_values = Vec::new();

        for entity in entities {
            let mut cols_values = Vec::new();
            for (name, field) in entity.fields() {
                if name != self.primary_key.0 || !self.primary_key.1 {
                    if cols_names.is_empty() {
                        cols_names.push(name);
                    }
                    let value = VC::convert(field.as_any());
                    cols_values.push(value);
                }
            }
            all_cols_values.push(cols_values);
        }

        Ok(InsertBuilder::into(self.table_name)
            .columns(&cols_names)
            .values(all_cols_values))
    }

    
    // Update operations
    pub fn update_by_key(&self, entity: T) -> Result<UpdateBuilder<D>, OperationError> {
        let mut cols_names = Vec::new();
        let mut cols_values = Vec::new();

        for (name, field) in entity.fields() {
            if name != self.primary_key.0 {
                if is_empty_or_none(field.as_any()) {
                    return Err(OperationError::new(format!("Field {} has an invalid value", name)));
                }
                cols_names.push(name);
                let value = VC::convert(field.as_any());
                cols_values.push(value);
            }
        }

        if cols_names.is_empty() {
            return Err(OperationError::new("No updatable fields provided".to_string()));
        }

        let primary_key_value = entity.fields()
            .find(|(name, _)| *name == self.primary_key.0)
            .map(|(_, field)| VC::convert(field.as_any()))
            .ok_or_else(|| OperationError::new(format!("Primary key {} not found", self.primary_key.0)))?;

        Ok(UpdateBuilder::table(self.table_name)
            .set_cols(&cols_names, cols_values)
            .where_(Expr::col(self.primary_key.0).eq(primary_key_value)))
    }

    pub fn update_one<F>(&self, entity: T, query_condition: Option<F>) -> Result<UpdateBuilder<D>, OperationError>
    where
        F: Fn(&mut UpdateBuilder<D>) + Send + 'a,
    {
        let mut cols_names = Vec::new();
        let mut cols_values = Vec::new();

        for (name, field) in entity.fields() {
            if name != self.primary_key.0 {
                let value = VC::convert(field.as_any());
                cols_names.push(name);
                cols_values.push(value);
            }
        }

        let primary_key_value = entity.fields()
            .find(|(name, _)| *name == self.primary_key.0)
            .map(|(_, field)| VC::convert(field.as_any()))
            .ok_or_else(|| OperationError::new("Primary key not found in entity".to_string()))?;

        let mut builder = UpdateBuilder::table(self.table_name)
            .set_cols(&cols_names, cols_values)
            .where_(Expr::col(self.primary_key.0).eq(primary_key_value));

        if let Some(condition) = query_condition {
            condition(&mut builder);
        }

        Ok(builder)
    }

    // Upsert operations
    pub fn upsert_one(&self, entity: T) -> Result<InsertBuilder<D>, OperationError> {
        let mut cols_names = Vec::new();
        let mut cols_values = Vec::new();
        let conflict_target = self.primary_key.0;

        for (name, field) in entity.fields() {
            if !cols_names.contains(&name) {
                cols_names.push(name);
            }

            let value = VC::convert(field.as_any());
            cols_values.push(value);
        } 

        Ok(InsertBuilder::into(self.table_name)
            .columns(&cols_names)
            .values(vec![cols_values])
            .on_conflict_do_update(conflict_target, &cols_names))
    }

    pub fn upsert_many(&self, entities: Vec<T>) -> Result<InsertBuilder<D>, OperationError> {
        if entities.is_empty() {
            return Err(OperationError::new("No entities provided for upsert operation".to_string()));
        }
    
        let mut cols_names = Vec::new();
        let mut all_cols_values = Vec::new();
        let conflict_target = self.primary_key.0;

        for (i, entity) in entities.iter().enumerate() {
            let mut cols_values = Vec::new();
    
            for (name, field) in entity.fields() {
                if i == 0 && !cols_names.contains(&name) {
                    cols_names.push(name);
                }

                let value = VC::convert(field.as_any());
                cols_values.push(value);
            }            
    
            all_cols_values.push(cols_values);
        }

        let mut builder: InsertBuilder<D> = InsertBuilder::into(self.table_name)
            .columns(&cols_names)
            .values(all_cols_values)
            .on_conflict_do_update(conflict_target, &cols_names);
    
        if self.primary_key.1 {
            builder = builder.returning(&[self.primary_key.0]);
        }

        Ok(builder)
    }


}