use crate::common::builder::BuilderTrait;
use super::{agg::Agg, case_when::WhenClause, filter::FilterClause, join::Join};
use std::fmt::Debug;

/// SQL builder, used to build the final SQL statement step by step.
#[derive(Debug, Clone)]
pub struct Builder <T: Debug + Clone> {
    sql: String,
    where_clauses: Vec<FilterClause<T>>,
    order_by_clauses: Vec<(String, bool)>,
    limit_offset: Option<(u64, Option<u64>)>,
    values: Vec<T>,
}

impl<T: Debug + Clone> BuilderTrait<T> for Builder<T> {
    type FilterClause = FilterClause<T>;
    type WhenClause<'a>  = WhenClause<'a, T>;
    type Join<'a> = Join<'a, T>;
    type Agg<'a>  = Agg<'a, T>;

    fn new(sql: impl Into<String>,  params: Option<Vec<T>>) -> Self {
        let values = params.unwrap_or(Vec::new());
        Self {
            sql: sql.into(),
            where_clauses: Vec::new(),
            order_by_clauses: Vec::new(),
            limit_offset: None,
            values,
        }
    }

    fn select(table: impl Into<String>, columns: &[&str]) -> Self {
        let mut sql = String::with_capacity(128);
        sql.push_str("SELECT ");
        if columns.is_empty() {
            sql.push('*');
        } else {
            sql.push_str(&columns.join(", "));
        }
        sql.push_str(" FROM ");
        sql.push_str(&table.into());
        Self::new(sql, None)
    }

    fn insert_into(table: &str, columns: &[&str], values: Vec<Vec<T>>) -> Self {
        let mut sql = String::with_capacity(128);
        sql.push_str("INSERT INTO ");
        sql.push_str(table);
        sql.push_str(" ( ");
        sql.push_str(&columns.join(", "));
        sql.push_str(" ) VALUES ");

        let mut cols_values = Vec::new();
        for row in values {
            sql.push('(');
            for (i, _) in row.iter().enumerate() {
                if i > 0 {
                    sql.push_str(", ");
                }
                sql.push('?');
            }
            sql.push(')');
            sql.push_str(", ");
            cols_values.extend(row);
        }

        // Remove the last extra comma and space
        if sql.ends_with(", ") {
            sql.truncate(sql.len() - 2);
        }

        Self::new(sql, Some(cols_values))
    }

    fn update(table: &str, columns: &[&str], values: Vec<T>) -> Self {
        let mut sql = String::with_capacity(128);
        sql.push_str("UPDATE ");
        sql.push_str(table);
        sql.push_str(" SET ");

        for (i, col) in columns.iter().enumerate() {
            if i > 0 {
                sql.push_str(", ");
            }
            sql.push_str(col);
            sql.push_str(" = ?");
        }

        Self::new(sql, Some(values))
    }

    fn delete(table: &str) -> Self {
        let mut sql = String::with_capacity(table.len() + 12);
        sql.push_str("DELETE FROM ");
        sql.push_str(table);
        Self::new(sql, None)
    }

    fn filter(&mut self, clause: FilterClause<T>) -> &mut Self {
        self.where_clauses.push(clause);
        self
    }

    fn and(&mut self, clause: FilterClause<T>) -> &mut Self {
        if let Some(last_clause) = self.where_clauses.pop() {
            let combined_clause = last_clause.and(clause);
            self.where_clauses.push(combined_clause);
        } else {
            self.where_clauses.push(clause);
        }
        self
    }

    fn or(&mut self, clause: FilterClause<T>) -> &mut Self {
        if let Some(last_clause) = self.where_clauses.pop() {
            let combined_clause = last_clause.or(clause);
            self.where_clauses.push(combined_clause);
        } else {
            self.where_clauses.push(clause);
        }
        self
    }

    fn order_by(&mut self, column: &str, asc: bool) -> &mut Self {
        // Check if the column already exists
        if let Some(index) = self
            .order_by_clauses
            .iter()
            .position(|(col, _)| col == column)
        {
            // If it exists, overwrite
            self.order_by_clauses[index] = (column.to_string(), asc);
        } else {
            // Otherwise, add a new sorting method
            self.order_by_clauses.push((column.to_string(), asc));
        }

        self
    }

    fn limit_offset(&mut self, limit: u64, offset: Option<u64>) -> &mut Self {
        self.limit_offset = Some((limit, offset));
        self
    }

    fn add_subquery(&mut self, builder: Self) -> &mut Self {
        let (query, params) = builder.build();
        self.sql.push('(');
        self.sql.push_str(&query);
        self.sql.push(')');
        self.values.extend(params);
        self
    }

    fn append(&mut self, sql: impl Into<String>, bind_values: Option<Vec<T>>) -> &mut Self {
        self.sql.push_str(&sql.into());
        if let Some(values) = bind_values {
            self.values.extend(values);
        }
        self
    }

    fn case_when(&mut self, case_when: WhenClause<T>) -> &mut Self {
        let (case_when_sql, case_when_values) = case_when.build();
        self.sql.push(' ');
        self.sql.push_str(&case_when_sql);
        self.values.extend(case_when_values);
        self
    }

    fn join(&mut self, join: Join<T>) -> &mut Self {
        let (join_sql, join_values) = join.build();
        self.sql.push_str(&join_sql);
        self.values.extend(join_values);
        self
    }

    fn aggregate(&mut self, agg: Agg<T>) -> &mut Self {
        let (agg_sql, agg_values) = agg.build("");
        self.sql.push_str(&agg_sql);
        self.values.extend(agg_values);
        self
    }

    fn build(self) -> (String, Vec<T>) {
        let mut sql = String::with_capacity(self.sql.len() + 256);
        let mut all_values = self.values;

        sql.push_str(&self.sql);

        // Add WHERE statement
        if !self.where_clauses.is_empty() {
            sql.push_str(" WHERE ");

            let mut first = true;
            for clause in self.where_clauses {
                if !first {
                    sql.push_str(" AND ");
                }
                let (clause_sql, clause_values) = clause.build();
                sql.push_str(&clause_sql);
                all_values.extend(clause_values);
                first = false;
            }
        }

        // Add ORDER BY statement
        if !self.order_by_clauses.is_empty() {
            sql.push_str(" ORDER BY ");

            let mut first = true;
            for (col, asc) in self.order_by_clauses {
                if !first {
                    sql.push_str(", ");
                }
                sql.push_str(&col);
                sql.push_str(if asc { " ASC" } else { " DESC" });
                first = false;
            }
        }

        // Add LIMIT and OFFSET statement
        if let Some((limit, offset)) = self.limit_offset {
            sql.push_str(" LIMIT ");
            sql.push_str(&limit.to_string());
            if let Some(offset) = offset {
                sql.push_str(" OFFSET ");
                sql.push_str(&offset.to_string());
            }
        }

        (sql, all_values)
    }

    fn build_mut(&mut self) -> (String, Vec<T>) {
        let mut sql = String::with_capacity(self.sql.len() + 128);
        let mut all_values = std::mem::take(&mut self.values);

        sql.push_str(&self.sql);

        // Add WHERE statement
        if !self.where_clauses.is_empty() {
            sql.push_str(" WHERE ");

            let mut first = true;
            for clause in self.where_clauses.drain(..) {
                if !first {
                    sql.push_str(" AND ");
                }
                let (clause_sql, clause_values) = clause.build();
                sql.push_str(&clause_sql);
                all_values.extend(clause_values);
                first = false;
            }
        }

        // Add ORDER BY statement
        if !self.order_by_clauses.is_empty() {
            sql.push_str(" ORDER BY ");

            let mut first = true;
            for (col, asc) in self.order_by_clauses.drain(..)  {
                if !first {
                    sql.push_str(", ");
                }
                sql.push_str(&col);
                sql.push_str(if asc { " ASC" } else { " DESC" });
                first = false;
            }
        }

        // Add LIMIT and OFFSET statement
        if let Some((limit, offset)) = self.limit_offset.take() {
            sql.push_str(" LIMIT ");
            sql.push_str(&limit.to_string());
            if let Some(offset) = offset {
                sql.push_str(" OFFSET ");
                sql.push_str(&offset.to_string());
            }
        }
        //dbg!(&sql);
        (sql, all_values)
    }
}
