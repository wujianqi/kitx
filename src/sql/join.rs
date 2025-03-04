use super::filter::FilterClause;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct Join<'a, T: Debug + Clone> {
    joins: Vec<(String, &'a str, Option<FilterClause<T>>)>, // (join_type, table, condition)
}

impl<'a, T: Debug + Clone> Join<'a, T> {
    /// Creates a new JOIN instance (private method, used internally)
    fn new(join_type: &str, table: &'a str) -> Self {
        Join {
            joins: vec![(join_type.to_string(), table, None)],
        }
    }

    /// Adds a LEFT JOIN
    pub fn left(mut self, table: &'a str) -> Self {
        if self.joins.is_empty() {
            Self::new("LEFT JOIN", table)
        } else {
            self.joins.push(("LEFT JOIN".to_string(), table, None));
            self
        }
    }

    /// Adds a RIGHT JOIN
    pub fn right(mut self, table: &'a str) -> Self {
        if self.joins.is_empty() {
            Self::new("RIGHT JOIN", table)
        } else {
            self.joins.push(("RIGHT JOIN".to_string(), table, None));
            self
        }
    }

    /// Adds an INNER JOIN
    pub fn inner(mut self, table: &'a str) -> Self {
        if self.joins.is_empty() {
            Self::new("INNER JOIN", table)
        } else {
            self.joins.push(("INNER JOIN".to_string(), table, None));
            self
        }
    }

    /// Adds a FULL OUTER JOIN
    pub fn full_outer(mut self, table: &'a str) -> Self {
        if self.joins.is_empty() {
            Self::new("FULL OUTER JOIN", table)
        } else {
            self.joins.push(("FULL OUTER JOIN".to_string(), table, None));
            self
        }
    }

    /// Sets the join condition for the last JOIN
    pub fn on(mut self, condition: FilterClause<T>) -> Self {
        if let Some(last_join) = self.joins.last_mut() {
            last_join.2 = Some(condition);
        }
        self
    }

    /// Builds SQL string and parameter values for all JOIN clauses
    pub fn build(&self) -> (String, Vec<T>)
    where
        T: Clone,
    {
        // Pre-allocate sufficient capacity
        let mut sql = String::with_capacity(self.joins.len() * 128); // Adjust capacity as needed
        let mut values = Vec::new();

        for (join_type, table, condition) in &self.joins {
            sql.push_str(" ");
            sql.push_str(join_type);
            sql.push_str(" ");
            sql.push_str(table);

            if let Some(condition) = condition {
                let (clause, condition_values) = condition.clone().build();
                sql.push_str(" ON ");
                sql.push_str(&clause);
                values.extend(condition_values);
            }
        }

        (sql, values)
    }
}