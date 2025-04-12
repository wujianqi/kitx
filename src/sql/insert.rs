use crate::common::builder::BuilderTrait;
use std::fmt::Debug;

use super::{cte::WithCTE, helper::build_returning_clause};

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
        for (row_idx, row) in values.into_iter().enumerate() {
            if row_idx > 0 {
                self.sql.push(',');
            }
            self.sql.push(' ');
            self.sql.push('(');
            for (i, val) in row.into_iter().enumerate() {
                if i > 0 {
                    self.sql.push_str(", ");
                }
                self.sql.push('?');
                cols_values.push(val);
            }
            self.sql.push(')');
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

    /// Adds a WITH clause to the SELECT statement.
    /// Supported in Mysql 8.0+、Sqlite 3.8.3+ only.
    pub fn with(mut self, with_cte: WithCTE<T>) -> Self {
        let (with_sql, with_values) = with_cte.build();
        let mut new_sql = String::with_capacity(with_sql.len() + self.sql.len());
        new_sql.push_str(&with_sql);
        new_sql.push_str(&self.sql);
        self.sql = new_sql;
        self.values.extend(with_values);
        self
    }

    /// NOTE: Supported in Mysql 8.0.21+、Sqlite 3.35+ only.
    pub fn returning(mut self, columns: &[&str]) -> Self {
        self.sql.push_str(&build_returning_clause(columns));
        self
    }

    /// Adds an `ON CONFLICT` clause with a `DO UPDATE` action.
    /// NOTE: Supported in Sqlite 3.24+ 、PostgreSQL、Mysql(`ON DUPLICATE`) only.
    #[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
    pub fn on_conflict_do_update(self, 
        conflict_target: &str, 
        excluded_columns: &[&str]
    ) -> Self {
        let mut sql = String::with_capacity(64);
    
        #[cfg(any(feature = "sqlite", feature = "postgres"))]
        {
            sql.push_str(" ON CONFLICT (");
            sql.push_str(conflict_target);
            sql.push_str(") DO UPDATE SET ");
        
            // Append the SET clause for excluded columns
            for (i, column) in excluded_columns.iter().enumerate() {
                if i > 0 {
                    sql.push_str(", ");
                }
                sql.push_str(column);
                sql.push_str(" = EXCLUDED.");
                sql.push_str(column);
            }
        }

        #[cfg(feature = "mysql")]
        {
            let _ = conflict_target;
            sql.push_str(" ON DUPLICATE KEY UPDATE ");

            for (i, column) in excluded_columns.iter().enumerate() {
                if i > 0 {
                    sql.push_str(", ");
                }
                sql.push_str(column);
                sql.push_str(" = VALUES(");
                sql.push_str(column);
                sql.push_str(")");
            }
        }
    
        self.append(sql, None)
    }
}

impl<T: Debug + Clone> BuilderTrait<T> for InsertBuilder<T> {
    /// Build method implementation for InsertBuilder, consuming self
    fn build(self) -> (String, Vec<T>) {
        (self.sql, self.values)
    }
}