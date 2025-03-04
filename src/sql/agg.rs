use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct Agg<'a, T: Debug + Clone> {
    aggregates: Vec<(&'a str, &'a str, Option<&'a str>)>, // (function, column, alias)
    group_by_columns: Vec<&'a str>,
    having_conditions: Vec<(&'a str, T)>, // Store HAVING conditions and bound values
    values: Vec<T>,
}

impl<'a, T: Debug + Clone> Agg<'a, T> {
    /// Creates a new `Agg` instance
    fn new() -> Self {
        Agg {
            aggregates: Vec::new(),
            group_by_columns: Vec::new(),
            having_conditions: Vec::new(),
            values: Vec::new(),
        }
    }

    /// Adds a COUNT aggregation function
    pub fn count(column: &'a str, alias: Option<&'a str>) -> Self {
        let mut agg = Self::new();
        agg.aggregates.push(("COUNT", column, alias));
        agg
    }

    /// Adds a SUM aggregation function
    pub fn sum(column: &'a str, alias: Option<&'a str>) -> Self {
        let mut agg = Self::new();
        agg.aggregates.push(("SUM", column, alias));
        agg
    }

    /// Adds an AVG aggregation function
    pub fn avg(column: &'a str, alias: Option<&'a str>) -> Self {
        let mut agg = Self::new();
        agg.aggregates.push(("AVG", column, alias));
        agg
    }

    /// Adds a MIN aggregation function
    pub fn min(column: &'a str, alias: Option<&'a str>) -> Self {
        let mut agg = Self::new();
        agg.aggregates.push(("MIN", column, alias));
        agg
    }

    /// Adds a MAX aggregation function
    pub fn max(column: &'a str, alias: Option<&'a str>) -> Self {
        let mut agg = Self::new();
        agg.aggregates.push(("MAX", column, alias));
        agg
    }

    /// Adds a GROUP BY clause
    pub fn group_by(mut self, columns: &[&'a str]) -> Self {
        self.group_by_columns.extend_from_slice(columns);
        self
    }

    /// Adds a HAVING condition and binds a value
    pub fn having(mut self, condition: &'a str, value: T) -> Self
    where
        T: Clone,
    {
        self.having_conditions.push((condition, value.clone()));
        self.values.push(value); // Store the original value
        self
    }

    /// Internal method: Adds aggregation functions to the SQL statement
    fn add_aggregates_to_sql(&self, sql: &mut String) {
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
    }

    /// Internal method: Adds HAVING conditions to the SQL statement
    fn add_having_to_sql(&self, sql: &mut String) {
        if !self.having_conditions.is_empty() {
            sql.push_str(" HAVING ");
            for (i, (condition, _)) in self.having_conditions.iter().enumerate() {
                if i > 0 {
                    sql.push_str(" AND ");
                }
                sql.push_str(condition);
                sql.push(' ');
                sql.push('?'); // Use placeholder
            }
        }
    }

    /// Builds the final SQL statement and parameter values
    pub fn build(self, base_sql: &str) -> (String, Vec<T>) {
        // Pre-allocate sufficient capacity
        let mut sql = String::with_capacity(base_sql.len() + self.aggregates.len() * 64 + self.group_by_columns.len() * 20 + self.having_conditions.len() * 30);

        sql.push_str(base_sql);

        // Add aggregation functions
        self.add_aggregates_to_sql(&mut sql);

        // Add GROUP BY clause
        if !self.group_by_columns.is_empty() {
            sql.push_str(" GROUP BY ");
            sql.push_str(&self.group_by_columns.join(", "));
        }

        // Add HAVING clause
        self.add_having_to_sql(&mut sql);

        (sql, self.values)
    }
}