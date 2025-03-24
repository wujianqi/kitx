use crate::common::builder::BuilderTrait;
use std::fmt::Debug;

use super::helper::build_returning_clause;

// INSERT-specific builder
#[derive(Default, Debug, Clone)]
pub struct InsertBuilder<T: Debug + Clone> {
    sql: String,
    values: Vec<T>,
}

impl<T: Debug + Clone> InsertBuilder<T> {

    /// Specifies the table for the INSERT statement.
    /// 
    /// # Parameters
    /// - `table`: Name of the table to insert into.
    ///
    /// # Returns
    /// - `InsertBuilder`: Initialized InsertBuilder instance.
    pub fn into(table: &str) -> Self {
        let mut sql = String::with_capacity(table.len() + 12);
        sql.push_str("INSERT INTO ");
        sql.push_str(table);
        Self {
            sql,
            values: vec![],
        }
    }

    /// Specifies the columns for the INSERT statement.
    /// 
    /// # Parameters
    /// - `columns`: List of column names to insert into.
    ///
    /// # Returns
    /// - `InsertBuilder`: Updated InsertBuilder instance.
    pub fn columns(mut self, columns: &[&str]) -> Self {
        self.sql.push_str(" (");
        for column in columns {
            if !self.sql.ends_with('(') {
                self.sql.push_str(", ");
            }
            self.sql.push_str(column);
        }
        self.sql.push_str(") ");
        self
    }

    /// Adds VALUES clause to the INSERT statement.
    /// 
    /// # Parameters
    /// - `values`: List of values to insert into the table.
    ///
    /// # Returns
    /// - `InsertBuilder`: Updated InsertBuilder instance.
    pub fn values(mut self, values: Vec<Vec<T>>) -> Self {
        self.sql.push_str("VALUES");
        let mut cols_values = Vec::new();
        for (row_idx, row) in values.iter().enumerate() {
            if row_idx > 0 {
                self.sql.push(',');
            }
            self.sql.push(' ');
            self.sql.push('(');
            for (i, _) in row.iter().enumerate() {
                if i > 0 {
                    self.sql.push_str(", ");
                }
                self.sql.push('?');
            }
            self.sql.push(')');
            cols_values.extend(row.clone());
        }

        if self.sql.ends_with(',') {
            self.sql.truncate(self.sql.len() - 1);
        }
        self.values = cols_values;
        self
    }

    
    /// Creates a new InsertBuilder instance with the given SQL query and parameter values.
    pub fn raw(sql: impl Into<String>, params: Option<Vec<T>>) -> Self {
        let sql = sql.into();
        let mut values = vec![];
        if let Some(vals) = params {
            values.extend(vals);
        }
        Self {
            sql,
            values,
        }        
    }

    /// Appends a new SQL query and parameter value to the existing query.
    pub fn append(mut self, sql: impl Into<String>, value: Option<T>)-> Self {
        let sql = sql.into();
        let mut values = vec![];
        if let Some(val) = value {
            values.push(val);
        }
        self.sql.push_str(&sql);
        self.values.extend(values);
        self
    }

    /// NOTE: Supported in PostgreSQL8.2+、Mysql 8.0.21+、Sqlite 3.35+ only.
    pub fn returning(mut self, columns: &[&str]) -> Self {
        self.sql.push_str(&build_returning_clause(columns));
        self
    }
}

impl<T: Debug + Clone> BuilderTrait<T> for InsertBuilder<T> {
    /// Build method implementation for InsertBuilder
    fn build(&self) -> (String, Vec<T>) {
        (self.sql.clone(), self.values.clone())
    }
}