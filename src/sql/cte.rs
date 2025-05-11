use std::fmt::Debug;
use crate::{common::builder::BuilderTrait, sql::select::SelectBuilder};

/// Represents a single Common Table Expression (CTE).
#[derive(Debug, Clone)]
pub struct CTE<T: Debug + Clone> {
    name: String,
    columns: Option<Vec<String>>,
    query: SelectBuilder<T>,
}

impl<T: Debug + Clone> CTE<T> {
    /// Creates a new CTE with the given name and subquery.
    pub fn new(name: impl Into<String>, query: SelectBuilder<T>) -> Self {
        CTE {
            name: name.into(),
            columns: None,
            query,
        }
    }

    /// Specifies the column names for the CTE.
    pub fn columns(mut self, columns: &[&str]) -> Self {
        self.columns = Some(columns.iter().map(|&col| col.to_string()).collect());
        self
    }

    /// Builds the SQL representation of this CTE.
    pub fn build(self) -> (String, Vec<T>) {
        let mut sql = String::with_capacity(self.name.len() + 32);
        sql.push_str(&self.name);
        
        if let Some(cols) = self.columns {
            sql.push('(');
            sql.push_str(&cols.join(", "));
            sql.push(')');
        }
        
        sql.push_str(" AS (");
        let (query_sql, query_values) = self.query.build();
        sql.push_str(&query_sql);
        sql.push_str(") ");
        
        (sql, query_values)
    }
}

/// Represents a collection of CTEs to be used in a WITH clause.
#[derive(Default, Debug, Clone)]
pub struct WithCTE<T: Debug + Clone> {
    ctes: Vec<CTE<T>>,
}

impl<T: Debug + Clone> WithCTE<T> {
    /// Creates a new empty WithCTE instance.
    pub fn new() -> Self {
        WithCTE { ctes: Vec::new() }
    }

    /// Adds a CTE to the collection.
    pub fn add_cte(&mut self, cte: CTE<T>) -> &mut Self {
        self.ctes.push(cte);
        self
    }

    /// Builds the SQL representation of all CTEs.
    pub fn build(self) -> (String, Vec<T>) {
        let mut sql = String::with_capacity(64);
        let mut values = Vec::new();

        if !self.ctes.is_empty() {
            sql.push_str("WITH ");
            for (i, cte) in self.ctes.into_iter().enumerate() {
                if i > 0 {
                    sql.push_str(", ");
                }
                let (cte_sql, cte_values) = cte.build();
                sql.push_str(&cte_sql);
                values.extend(cte_values);
            }
        }

        (sql, values)
    }
}