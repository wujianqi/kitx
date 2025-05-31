use std::fmt::Debug;

use field_access::FieldAccess;
use sqlx::{Database, Error, FromRow};

use crate::{
    common::error::QueryError, 
    sql::{filter::Expr, insert::InsertBuilder, update::UpdateBuilder}, 
    utils::type_conversion::{is_none, ValueConvert}
};

use super::single::SingleKeyTable;

impl<'a, T, D, DB, VC> SingleKeyTable<'a, T, D, DB, VC>
where
    T: for<'r> FromRow<'r, DB::Row> + FieldAccess + Unpin + Send + Sync + Default,
    D: Clone + Debug + Send + Sync + 'a,
    DB: Database,
    VC: ValueConvert<D>,
{
    fn get_primary_key_value(&self, entity: &T) -> Result<D, Error> {
        entity.fields()
            .find(|(name, _)| *name == self.primary.0)
            .map(|(_, field)| VC::convert(field.as_any()))
            .ok_or_else(|| QueryError::PrimaryKeyNotFound(self.primary.0.to_string()).into())
    }

    // Insert operations
    pub fn insert_one(&self, entity: T) -> Result<InsertBuilder<D>, Error> {
        let mut cols_names = Vec::new();
        let mut cols_values = Vec::new();

        for (name, field) in entity.fields() {
            if name != self.primary.0 || !self.primary.1 {
                if is_none(field.as_any()) {
                    return Err(QueryError::ValueInvalid(name.to_string()).into());
                }
                cols_names.push(name);
                let value = VC::convert(field.as_any());
                cols_values.push(value);
            }
        }

        if cols_names.is_empty() {
            return Err(QueryError::ColumnsListEmpty.into());
        }

        Ok(InsertBuilder::into(self.table_name)
            .columns(&cols_names)
            .values(vec![cols_values]))
    }

    pub fn insert_many(&self, entities: Vec<T>) -> Result<InsertBuilder<D>, Error> {
        if entities.is_empty() {
            return Err(QueryError::NoEntitiesProvided.into());
        }

        let mut cols_names = Vec::new();
        let mut all_cols_values = Vec::new();

        for entity in entities {
            let mut cols_values = Vec::new();
            for (name, field) in entity.fields() {
                if name != self.primary.0 || !self.primary.1 {
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
    pub fn update_by_key(&self, entity: T) -> Result<UpdateBuilder<D>, Error> {
        let mut cols_names = Vec::new();
        let mut cols_values = Vec::new();
        let pk_name = self.primary.0;

        for (name, field) in entity.fields() {
            if name != pk_name {
                if is_none(field.as_any()) {
                    return Err(QueryError::ValueInvalid(name.to_string()).into());
                }
                cols_names.push(name);
                let value = VC::convert(field.as_any());
                cols_values.push(value);
            }
        }

        if cols_names.is_empty() {
            return Err(QueryError::ColumnsListEmpty.into());
        }

        let primary_key_value = self.get_primary_key_value(&entity)?;

        Ok(UpdateBuilder::table(self.table_name)
            .set_cols(&cols_names, cols_values)
            .and_where(Expr::col(pk_name).eq(primary_key_value)))
    }

    pub fn update_by_expr<F>(
        &self,
        columns: &[(&str, &str)],
        condition: F,
    ) -> Result<UpdateBuilder<D>, Error>
    where
        F: Fn(&mut UpdateBuilder<D>) + Send + 'a,
    {
        if columns.is_empty() {
            return Err(QueryError::ColumnsListEmpty.into());
        }
    
        let mut builder = UpdateBuilder::table(self.table_name);
    
        for (col, expr) in columns {
            builder = builder.set_expr(col, expr);
        }
    
        condition(&mut builder);
    
        Ok(builder)
    }


    pub fn update_one<F>(&self, entity: T, query_condition: F) -> Result<UpdateBuilder<D>, Error>
    where
        F: Fn(&mut UpdateBuilder<D>) + Send + 'a,
    {
        let pk_name  = self.primary.0;
        let mut cols_names = Vec::new();
        let mut cols_values = Vec::new();

        for (name, field) in entity.fields() {
            if name != pk_name {
                let value = VC::convert(field.as_any());
                cols_names.push(name);
                cols_values.push(value);
            }
        }

        let primary_key_value = self.get_primary_key_value(&entity)?;

        let mut builder = UpdateBuilder::table(self.table_name)
            .set_cols(&cols_names, cols_values)
            .and_where(Expr::col(pk_name).eq(primary_key_value));

        query_condition(&mut builder);

        Ok(builder)
    }

    // Upsert operations
    pub fn upsert_one(&self, entity: T) -> Result<InsertBuilder<D>, Error> {
        let pk_name = self.primary.0;
        let mut cols_names = Vec::new();
        let mut cols_values = Vec::new();

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
            .on_conflict_do_update(pk_name, &cols_names))
    }

    pub fn upsert_many(&self, entities: Vec<T>) -> Result<InsertBuilder<D>, Error> {
        if entities.is_empty() {
            return Err(QueryError::NoEntitiesProvided.into());
        }

        let pk_name = self.primary.0;
        let mut cols_names = Vec::new();
        let mut all_cols_values = Vec::new();

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

        let builder: InsertBuilder<D> = InsertBuilder::into(self.table_name)
            .columns(&cols_names)
            .values(all_cols_values)
            .on_conflict_do_update(pk_name, &cols_names);

        Ok(builder)
    }

}