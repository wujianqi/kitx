use std::{fmt::Debug, mem::take};
use super::filter::Expr;

#[derive(Debug, Clone, Default)]
pub struct Func<T: Debug + Clone> {
    aggregates: Vec<(String, String, String)>, // (function, column, alias)
    group_by_columns: Vec<String>,
    having_conditions: Vec<Expr<T>>, // Store HAVING conditions and bound values
}

impl<T: Debug + Clone> Func<T> {
    /// Adds a COUNT aggregation function
    /// # Parameters
    /// - `column`: Column to count
    /// - `alias`: Alias for the count result
    ///
    /// # Returns
    /// - `Func`: Updated Agg instance.
    pub fn count(mut self, column: &str, alias: &str) -> Self {
        self.aggregates.push(("COUNT".into(), column.into(), alias.into()));
        self
    }

    /// Adds a SUM aggregation function
    /// # Parameters
    /// - `column`: Column to sum
    /// - `alias`: Alias for the sum result
    ///
    /// # Returns
    /// - `Func`: Updated Agg instance.
    pub fn sum(mut self, column: &str, alias: &str) -> Self {
        self.aggregates.push(("SUM".into(), column.into(), alias.into()));
        self
    }

    /// Adds an AVG aggregation function
    /// # Parameters
    /// - `column`: Column to average
    /// - `alias`: Alias for the average result
    ///
    /// # Returns
    /// - `Func`: Updated Agg instance.
    pub fn avg(mut self, column: &str, alias: &str) -> Self {
        self.aggregates.push(("AVG".into(), column.into(), alias.into()));
        self
    }

    /// Adds a MIN aggregation function
    /// # Parameters
    /// - `column`: Column to find the minimum value of
    /// - `alias`: Alias for the minimum value
    ///
    /// # Returns
    /// - `Func`: Updated Agg instance.
    pub fn min(mut self, column: &str, alias: &str) -> Self {
        self.aggregates.push(("MIN".into(), column.into(), alias.into()));
        self
    }

    /// Adds a MAX aggregation function
    /// # Parameters
    /// - `column`: Column to find the maximum value of
    /// - `alias`: Alias for the maximum value
    ///
    /// # Returns
    /// - `Func`: Updated Agg instance.
    pub fn max(mut self, column: &str, alias: &str) -> Self {
        self.aggregates.push(("MAX".into(), column.into(), alias.into()));
        self
    }

    /// Adds a GROUP BY clause
    /// # Parameters
    /// - `columns`: Columns to group by
    pub fn group_by(mut self, columns: &[&str]) -> Self {
        self.group_by_columns.extend(columns.iter().map(|c| c.to_string()));
        self
    }

    /// Adds a HAVING condition and binds a value
    /// # Parameters
    /// - `condition`: Condition to add to the HAVING clause
    pub fn having(mut self, condition: Expr<T>) -> Self
    where
        T: Clone,
    {
        self.having_conditions.push(condition);
        self
    }

    /// Adds an AND condition to the existing having
    pub fn and(mut self, condition: Expr<T>) -> Self 
    where 
        T: Default
    {
        if let Some(existing_filter) = self.having_conditions.last_mut() {
            *existing_filter = take(existing_filter).and(condition);
        } else {
            self.having_conditions.push(condition);
        }
        self
    }

    /// Adds an OR condition to the existing filter
    pub fn or(mut self, condition: Expr<T>) -> Self 
    where 
        T: Default
    {
        if let Some(existing_filter) = self.having_conditions.last_mut() {
            *existing_filter = take(existing_filter).or(condition);
        } else {
            self.having_conditions.push(condition);
        }
        self
    }

    /// Builds the SQL for the aggregation functions
    pub fn build_aggregates(&self) -> String {
        let mut sqls = Vec::new();
        // Add aggregation functions
        for (func, column, alias) in &self.aggregates {
            let mut strs = String::with_capacity(30);
            strs.push_str(func);
            strs.push('(');
            strs.push_str(column);
            strs.push(')');

            if !alias.is_empty() {
                strs.push_str(" AS ");
                strs.push_str(alias);
            }
            sqls.push(strs);
        }
        sqls.join(", ")
    }

    /// Builds the SQL for the GROUP BY and HAVING clauses
    pub fn build_group_having(self) -> Option<(String, Vec<T>)> {
        if self.group_by_columns.is_empty() && self.having_conditions.is_empty() {
            return None;
        }
        let mut sql = String::with_capacity(128);
        let mut all_values = Vec::new();

        // Add GROUP BY clause
        if !self.group_by_columns.is_empty() {
            sql.push_str(" GROUP BY ");
            sql.push_str(&self.group_by_columns.join(", "));
        }

        // Add HAVING clause and extract parameter values
        if !self.having_conditions.is_empty() {
            sql.push_str(" HAVING ");
            let mut first = true;
            for clause in self.having_conditions {
                if !first {
                    sql.push_str(" AND ");
                }
                let (clause_sql, clause_values) = clause.build();
                sql.push_str(&clause_sql);
                all_values.extend(clause_values);
                first = false;
            }
        }

        Some((sql, all_values))
    }

}