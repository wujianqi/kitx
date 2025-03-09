use super::filter::FilterClause;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct Join<'a, T: Debug + Clone> {
    join_type: (&'a str, &'a str), // (join_type, table,)
    filter: Option<FilterClause<T>>,
}

impl<'a, T: Debug + Clone> Join<'a, T> {
    /// Creates a new JOIN instance (private method, used internally)
    fn new(join_type: &'a str, table: &'a str) -> Self {
        Join {
            join_type: (join_type, table),
            filter: None,
        }
    }

    /// Adds a LEFT JOIN
    pub fn left(table: &'a str) -> Self {
        Self::new("LEFT JOIN", table)
    }

    /// Adds a RIGHT JOIN
    pub fn right(table: &'a str) -> Self {
        Self::new("RIGHT JOIN", table)
    }

    /// Adds an INNER JOIN
    pub fn inner(table: &'a str) -> Self {
        Self::new("INNER JOIN", table)
    }

    /// Adds a FULL OUTER JOIN
    pub fn full_outer(table: &'a str) -> Self {
        Self::new("FULL OUTER JOIN", table)
    }

    /// Adds a CROSS JOIN
    pub fn cross(table: &'a str) -> Self {
        Self::new("CROSS JOIN", table)
    }

    /// Sets the join condition for the last JOIN
    pub fn on(mut self, condition: FilterClause<T>) -> Self {
        self.filter = Some(condition);
        self
    }

     /// Adds an AND condition to the existing filter
     pub fn and(mut self, condition: FilterClause<T>) -> Self {
        if let Some(existing_filter) = self.filter {
            self.filter = Some(existing_filter.and(condition));
        } else {
            self.filter = Some(condition);
        }
        self
    }

    /// Adds an OR condition to the existing filter
    pub fn or(mut self, condition: FilterClause<T>) -> Self {
        if let Some(existing_filter) = self.filter {
            self.filter = Some(existing_filter.or(condition));
        } else {
            self.filter = Some(condition);
        }
        self
    }

    /// Builds SQL string and parameter values for all JOIN clauses
    pub fn build(&self) -> (String, Vec<T>)
    where
        T: Clone,
    {
        // Pre-allocate sufficient capacity for SQL string
        let mut sql = String::with_capacity(128);
        let mut values = Vec::new();

        // Destructure join_type to get join type and table name
        let (join_type, table) = self.join_type;

        // Append JOIN clause
        sql.push_str(" ");
        sql.push_str(join_type);
        sql.push_str(" ");
        sql.push_str(table);

        // Append ON condition if filter exists
        if let Some(filter) = &self.filter {
            sql.push_str(" ON ");
            let (clause, condition_values) = filter.clone().build();
            sql.push_str(&clause);
            values.extend(condition_values);
        }

        (sql, values)
    }
}