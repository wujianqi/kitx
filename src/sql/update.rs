use std::collections::HashMap;
use std::fmt::Debug;

use crate::common::builder::{BuilderTrait, FilterTrait};

use super::case_when::CaseWhen;
use super::cte::WithCTE;
use super::filter::Expr;
use super::helper::{build_returning_clause, build_where_clause, combine_where_clause};
use super::join::JoinType;

#[derive(Debug, Clone)]
enum ColumnUpdate<T: Debug + Clone> {
    Value(T),
    Expr(String),
}

// UPDATE-specific builder
#[derive(Default, Debug, Clone)]
pub struct UpdateBuilder<T: Debug + Clone> {
    sql: String,
    values: Vec<T>,
    columns: HashMap<String, ColumnUpdate<T>>,
    where_clauses: Vec<Expr<T>>,
    joins: Vec<JoinType<T>>,
}

impl<T: Debug + Clone> UpdateBuilder<T> {
    /// Specifies the table to be updated.
    /// 
    /// # Parameters
    /// - `table`: Name of the table to be updated.
    ///
    /// # Returns
    /// - `UpdateBuilder`: Initialized UpdateBuilder instance.
    pub fn table(table: &str) -> Self {
        let mut sql = String::with_capacity(table.len() + 12);
        sql.push_str("UPDATE ");
        sql.push_str(table);
        sql.push_str(" SET ");
        Self {
            sql,
            values: vec![],
            columns: HashMap::new(),
            where_clauses: vec![],
            joins: vec![],
        }
    }

    /// Sets a value for a column in the UPDATE statement.
    pub fn set(mut self, column: &str, value: T) -> Self {
        self.set_mut(column, value);
        self
    }

    /// Sets a value for a column in the UPDATE statement.
    pub fn set_mut(&mut self, column: &str, value: T) -> &mut Self {
        self.columns.insert(column.to_string(), ColumnUpdate::Value(value));
        self
    }

    /// Sets an expression for a column in the UPDATE statement.
    pub fn set_expr(mut self, column: &str, expr_sql: &str) -> Self {
        self.set_expr_mut(column, expr_sql);
        self
    }

    pub fn set_expr_mut(&mut self, column: &str, expr_sql: &str) -> &mut Self {
        self.columns.insert(column.to_string(), ColumnUpdate::Expr(expr_sql.to_string()));
        self
    }

    /// Specifies the columns to be updated and their corresponding values.
    ///
    /// # Parameters
    /// - `columns`: List of column names to be updated.
    /// - `values`: List of values to be updated.
    ///
    /// # Returns
    /// - `UpdateBuilder`: Updated UpdateBuilder instance.
    ///
    /// # Panics
    /// - If the number of columns does not match the number of values.
    pub fn set_cols(mut self, columns: &[&str], values: Vec<T>) -> Self {
        if columns.len() == values.len() {
            for (col, value) in columns.iter().zip(values.into_iter()) {
                self.columns.insert((*col).to_string(), ColumnUpdate::Value(value));
            }
        }
        self
    }

    /// Adds an AND condition to the last WHERE clause.
    /// 
    /// # Parameters
    /// - `filter`: AND condition to be added.
    ///
    /// # Returns
    /// - `UpdateBuilder`: Updated UpdateBuilder instance.
    pub fn and_where(mut self, filter: Expr<T>) -> Self {
        self.and_where_mut(filter);
        self
    }

    /// Adds an OR condition to the last WHERE clause.
    /// 
    /// # Parameters
    /// - `filter`: OR condition to be added.
    ///
    /// # Returns
    /// - `UpdateBuilder`: Updated UpdateBuilder instance.
    pub fn or_where(mut self, filter: Expr<T>) -> Self {
        self.or_where_mut(filter);
        self
    }

    /// Adds a JOIN clause to the UPDATE statement.
    pub fn join(mut self, join_clauses: JoinType<T>) -> Self {
        self.join_mut(join_clauses);
        self
    }

    /// Adds a JOIN clause to the UPDATE statement.
    pub fn join_mut(&mut self, join_clauses: JoinType<T>) -> &mut Self {
        self.joins.push(join_clauses);
        self
    }

    /// Adds a CASE WHEN clause to the UPDATE statement.
    pub fn case_when(mut self, case_when: CaseWhen<T>) -> Self {
        self.case_when_mut(case_when);
        self
    }

    /// Adds a CASE WHEN clause to the UPDATE statement.
    pub fn case_when_mut(&mut self, case_when: CaseWhen<T>) -> &mut Self {
        let (case_when_sql, case_when_values) = case_when.build();
        self.sql.push_str(", ");
        self.sql.push_str(&case_when_sql);
        self.values.extend(case_when_values);
        self
    }

    /// NOTE: Supported in PostgreSQL8.2+、Mysql 8.0.21+、Sqlite 3.35+ only.
    pub fn returning(mut self, columns: &[&str]) -> Self {
        self.sql.push_str(&build_returning_clause(columns));
        self
    }

    /// Appends a new SQL query and parameter value to the existing query.
    pub fn append(mut self, sql: impl Into<String>, value: Vec<T>)-> Self {
        self.append_mut(sql, value);
        self
    }

    /// Appends a new SQL query and parameter value to the existing query.
    pub fn append_mut(&mut self, sql: impl Into<String>, value: Vec<T>)-> &mut Self {
        let sql = sql.into();
        
        self.sql.push_str(&sql);
        if !value.is_empty() {
            self.values.extend(value);
        }
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

    /// Returns a reference to the WHERE clauses.
    pub fn take_where_clauses(self) -> Vec<Expr<T>> {
        self.where_clauses
    }
    

}

impl<T: Debug + Clone> FilterTrait<T> for UpdateBuilder<T> {
    type Expr = Expr<T>;
    /// Adds an AND condition to the last WHERE clause.
    fn and_where_mut<F>(&mut self, filter: F) -> &mut Self
    where
        F: Into<Self::Expr>
    {
        combine_where_clause(&mut self.where_clauses, filter.into(), false);
        self
    }

    /// Adds an OR condition to the last WHERE clause.
    fn or_where_mut<F>(&mut self, filter: F) -> &mut Self
    where
        F: Into<Self::Expr>
    {
        combine_where_clause(&mut self.where_clauses, filter.into(), true);
        self
    }
}

impl<T: Debug + Clone> BuilderTrait<T> for UpdateBuilder<T> {    
    /// Builds the UPDATE statement and returns the SQL query string and parameter values.
    fn build(self) -> (String, Vec<T>) {
        let mut sql = self.sql;
        let mut values = self.values;

        if !self.columns.is_empty() {
            let mut first = true;
            let cols: Vec<_> = self.columns.into_iter().collect();
            //cols.sort_by(|a, b| a.0.cmp(&b.0));

            for (col, update) in &cols {
                if !first {
                    sql.push_str(", ");
                }
                first = false;
                sql.push_str(&col);
                sql.push_str(" = ");
                match update {
                    ColumnUpdate::Value(_) => sql.push('?'),
                    ColumnUpdate::Expr(expr) => sql.push_str(&expr),
                }
            }

            for (_, update) in &cols {
                if let ColumnUpdate::Value(value) = update {
                    values.push(value.clone());
                }
            }
        }

        if !self.where_clauses.is_empty() {
            let (where_sql, where_values) = build_where_clause(self.where_clauses);
            sql.push(' ');
            sql.push_str(&where_sql);
            values.extend(where_values);
        }

        (sql, values)
    }
}