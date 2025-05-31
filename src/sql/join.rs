use super::filter::Expr;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct JoinType<T: Debug + Clone> {
    join_type: (String, String), // (join_type, table,)
    on_filter: Option<Expr<T>>,
}

impl<T: Debug + Clone> JoinType<T> {
    /// Creates a new JOIN instance (private method, used internally)
    fn new(join_type: impl Into<String>, table: impl Into<String>) -> Self {
        JoinType {
            join_type: (join_type.into(), table.into()),
            on_filter: None,
        }
    }

    /// Adds a LEFT JOIN
    pub fn left(table: impl Into<String>) -> Self {
        Self::new("LEFT JOIN", table)
    }

    /// Adds a RIGHT JOIN
    pub fn right(table: impl Into<String>) -> Self {
        Self::new("RIGHT JOIN", table)
    }

    /// Adds an INNER JOIN
    pub fn inner(table: impl Into<String>) -> Self {
        Self::new("INNER JOIN", table)
    }

    /// Adds a FULL OUTER JOIN
    pub fn full_outer(table: impl Into<String>) -> Self {
        Self::new("FULL OUTER JOIN", table)
    }

    /// Adds a CROSS JOIN
    pub fn cross(table: impl Into<String>) -> Self {
        Self::new("CROSS JOIN", table)
    }

    /// Sets the join condition for the last JOIN
    pub fn on(mut self, condition: Expr<T>) -> Self {
        self.on_filter = Some(condition);
        self
    }

    /// Adds an AND condition to the existing filter
    pub fn and(mut self, condition: Expr<T>) -> Self {
        if let Some(existing_filter) = self.on_filter {
            self.on_filter = Some(existing_filter.and(condition));
        } else {
            self.on_filter = Some(condition);
        }
        self
    }

    /// Adds an OR condition to the existing filter
    pub fn or(mut self, condition: Expr<T>) -> Self {
        if let Some(existing_filter) = self.on_filter {
            self.on_filter = Some(existing_filter.or(condition));
        } else {
            self.on_filter = Some(condition);
        }
        self
    }

    /// Builds SQL string and parameter values for all JOIN clauses
    pub fn build(self) -> (String, Vec<T>)
    where
        T: Clone,
    {
        // Pre-allocate sufficient capacity for SQL string
        let mut sql = String::with_capacity(128);
        let mut values = Vec::new();

        // Destructure join_type to get join type and table name
        let (join_type, table) = &self.join_type;

        // Append JOIN clause
        // sql.push_str(" ");
        sql.push_str(join_type);
        sql.push_str(" ");
        sql.push_str(table);

        // Append ON condition if filter exists
        if let Some(filter) = self.on_filter {
            sql.push_str(" ON ");
            let (clause, condition_values) = filter.build();
            sql.push_str(&clause);
            values.extend(condition_values);
        }

        (sql, values)
    }
}