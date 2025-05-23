use std::fmt::Debug;

use crate::common::builder::{BuilderTrait, FilterTrait};

use super::filter::Expr;
use super::helper::{build_returning_clause, build_where_clause, combine_where_clause};

// DELETE-specific builder
#[derive(Default, Debug, Clone)]
pub struct DeleteBuilder<T: Debug + Clone> {
    sql: String,
    where_clauses: Vec<Expr<T>>,
}

impl<T: Debug + Clone> DeleteBuilder<T> {  
    /// Specifies the table for the DELETE statement.
    /// 
    /// # Parameters
    /// - `table`: Name of the table to delete from.
    /// 
    /// # Returns
    /// - `DeleteBuilder`: Initialized DeleteBuilder instance.
    pub fn from(table: &str) -> Self {
        let mut sql = String::with_capacity(table.len() + 12);
        sql.push_str("DELETE FROM ");
        sql.push_str(table);
        Self {
            sql,
            where_clauses: vec![],
        }
    }
    
    /// Adds an AND condition to the last WHERE clause.
    /// 
    /// # Parameters
    /// - `filter`: AND condition expression.
    pub fn and_where(mut self, filter: Expr<T>) -> Self {
        self.and_where_mut(filter);
        self
    }

    /// Adds an OR condition to the last WHERE clause.
    /// 
    /// # Parameters
    /// - `filter`: OR condition expression.
    pub fn or_where(mut self, filter: Expr<T>) -> Self {
        self.or_where_mut(filter);
        self
    }

    /// Adds a RETURNING clause to the DELETE statement.
    /// NOTE: Supported in PostgreSQL8.2+、Mysql 8.0.21+、Sqlite 3.35+ only.
    pub fn returning(mut self, columns: &[&str]) -> Self {
        self.sql.push_str(&build_returning_clause(columns));
        self
    }

    /// Returns a reference to the WHERE clauses.
    pub fn take_where_clauses(self) -> Vec<Expr<T>> {
        self.where_clauses
    }
}

impl<T: Debug + Clone> FilterTrait<T> for DeleteBuilder<T> {
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

impl<T: Debug + Clone> BuilderTrait<T> for DeleteBuilder<T> {
    fn build(self) -> (String, Vec<T>) {
        let mut sql = self.sql;
        let mut values = vec![];

        // Process WHERE clauses
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