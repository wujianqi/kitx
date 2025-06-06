use std::fmt::Debug;

use crate::common::builder::{BuilderTrait, FilterTrait};

use super::case_when::CaseWhen;
use super::cte::WithCTE;
use super::filter::Expr;
use super::helper::{build_returning_clause, build_where_clause, combine_where_clause};
use super::join::JoinType;

// UPDATE-specific builder
#[derive(Default, Debug, Clone)]
pub struct UpdateBuilder<T: Debug + Clone> {
    sql: String,
    values: Vec<T>,
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
            where_clauses: vec![],
            joins: vec![],
        }
    }

    pub fn set(mut self, column: &str, value: T) -> Self {
        let mut capacity = column.len() + 4; // column name + " = ?"
        if !self.values.is_empty() {
            capacity += 1; // comma
        }
        self.sql.reserve(capacity);

        if !self.values.is_empty() {
            self.sql.push_str(", ");
        }
        self.sql.push_str(column);
        self.sql.push_str(" = ?");
        self.values.push(value);
        self
    }

    pub fn set_expr(mut self, column: &str, expr_sql: &str) -> Self {
        let mut capacity = column.len() + expr_sql.len() + 3; // "col = expr"
        if !self.values.is_empty() {
            capacity += 2; // for ", "
        }
        self.sql.reserve(capacity);
    
        if !self.values.is_empty() {
            self.sql.push_str(", ");
        }
        self.sql.push_str(column);
        self.sql.push_str(" = ");
        self.sql.push_str(expr_sql);
    
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
         let mut capacity = 0;
        for col in columns {
            capacity += col.len() + 4; // column name + " = ?"
        }
        self.sql.reserve(capacity);

        for (i, col) in columns.iter().enumerate() {
            if i > 0 {
                self.sql.push_str(", ");
            }
            self.sql.push_str(col);
            self.sql.push_str(" = ?");
        }

        self.values = values;
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

    pub fn join_mut(&mut self, join_clauses: JoinType<T>) -> &mut Self {
        self.joins.push(join_clauses);
        self
    }

    /// Adds a CASE WHEN clause to the UPDATE statement.
    pub fn case_when(mut self, case_when: CaseWhen<T>) -> Self {
        self.case_when_mut(case_when);
        self
    }

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
    pub fn append(mut self, sql: impl Into<String>, value: Option<T>)-> Self {
        self.append_mut(sql, value);
        self
    }

    pub fn append_mut(&mut self, sql: impl Into<String>, value: Option<T>)-> &mut Self {
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

        if !self.where_clauses.is_empty() {
            let (where_sql, where_values) = build_where_clause(self.where_clauses);
            if !sql.ends_with(' ') {
                sql.push(' ');
            }
            sql.push_str(&where_sql);
            values.extend(where_values);
        }

        (sql, values)
    }
}