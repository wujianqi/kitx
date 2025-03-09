use std::fmt::Debug;
use super::filter::FilterClause;

#[derive(Debug, Clone, Default)]
pub struct Agg<'a, T: Debug + Clone> {
    aggregates: Vec<(&'a str, &'a str, Option<&'a str>)>, // (function, column, alias)
    group_by_columns: Vec<&'a str>,
    having_conditions: Vec<FilterClause<T>>, // Store HAVING conditions and bound values
}

impl<'a, T: Debug + Clone> Agg<'a, T> {
    /// Adds a COUNT aggregation function
    pub fn count(mut self, column: &'a str, alias: Option<&'a str>) -> Self {
        self.aggregates.push(("COUNT", column, alias));
        self
    }

    /// Adds a SUM aggregation function
    pub fn sum(mut self, column: &'a str, alias: Option<&'a str>) -> Self {
        self.aggregates.push(("SUM", column, alias));
        self
    }

    /// Adds an AVG aggregation function
    pub fn avg(mut self, column: &'a str, alias: Option<&'a str>) -> Self {
        self.aggregates.push(("AVG", column, alias));
        self
    }

    /// Adds a MIN aggregation function
    pub fn min(mut self, column: &'a str, alias: Option<&'a str>) -> Self {
        self.aggregates.push(("MIN", column, alias));
        self
    }

    /// Adds a MAX aggregation function
    pub fn max(mut self, column: &'a str, alias: Option<&'a str>) -> Self {
        self.aggregates.push(("MAX", column, alias));
        self
    }

    /// Adds a GROUP BY clause
    pub fn group_by(mut self, columns: &[&'a str]) -> Self {
        self.group_by_columns.extend_from_slice(columns);
        self
    }

    /// Adds a HAVING condition and binds a value
    pub fn having(mut self, condition: FilterClause<T>) -> Self
    where
        T: Clone,
    {
        self.having_conditions.push(condition);
        self
    }

    /// Adds an AND condition to the existing having
    pub fn and(mut self, condition: FilterClause<T>) -> Self {
        if let Some(existing_filter) = self.having_conditions.last_mut() {
            *existing_filter = existing_filter.clone().and(condition);
        } else {
            self.having_conditions.push(condition);
        }
        self
    }

    /// Adds an OR condition to the existing filter
    pub fn or(mut self, condition: FilterClause<T>) -> Self {
        if let Some(existing_filter) = self.having_conditions.last_mut() {
            *existing_filter = existing_filter.clone().or(condition);
        } else {
            self.having_conditions.push(condition);
        }
        self
    }


    /// Builds the final SQL statement and parameter values
    pub fn build(self, base_sql: &str) -> (String, Vec<T>) {
        // Pre-allocate sufficient capacity
        let mut sql = String::with_capacity(256);    
        sql.push_str(base_sql);
    
        // Add aggregation functions
        for (func, column, alias) in &self.aggregates {
            sql.push_str(", ");
            sql.push_str(func);
            sql.push('(');
            sql.push_str(column);
            sql.push(')');
            if let Some(alias) = alias {
                sql.push_str(" AS ");
                sql.push_str(alias);
            }
        }
    
        // Add GROUP BY clause
        if !self.group_by_columns.is_empty() {
            sql.push_str(" GROUP BY ");
            sql.push_str(&self.group_by_columns.join(", "));
        }
    
        // Add HAVING clause and extract parameter values
        let mut all_values = Vec::new(); // 初始化参数值列表
        if !self.having_conditions.is_empty() {
            sql.push_str(" HAVING ");
            let mut first = true;
            for clause in self.having_conditions {
                if !first {
                    sql.push_str(" AND ");
                }
                let (clause_sql, clause_values) = clause.build();
                sql.push_str(&clause_sql);
                all_values.extend(clause_values); // 合并参数值
                first = false;
            }
        }
    
        (sql, all_values)
    }
}