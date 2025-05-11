use std::{collections::HashMap, fmt::Debug};
use crate::common::types::OrderBy;

use super::filter::Expr;

// Helper method to build WHERE clause
pub fn build_where_clause<T: Debug + Clone>(where_clauses: Vec<Expr<T>>) -> (String, Vec<T>) {
    if where_clauses.is_empty() {
        return (String::with_capacity(128), Vec::new());
    }

    let mut final_sql = String::with_capacity(100);
    let mut values = Vec::new();

    for (_, clause) in where_clauses.into_iter().enumerate() {
        let (clause_sql, clause_values) = clause.build();
        /* if i > 0 {
            final_sql.push_str(" AND ");
        } */
        final_sql.push_str(&clause_sql);
        values.extend(clause_values);
    }

    final_sql.insert_str(0, "WHERE ");
    (final_sql, values)
}

/// Combines multiple WHERE clauses into a single WHERE clause.
pub fn combine_where_clause<T: Debug + Clone>(clauses: &mut Vec<Expr<T>>, filter: Expr<T>, is_or: bool) {
    if let Some(last_clause) = clauses.pop() {
        let combined_clause = if is_or {
            last_clause.or(filter)
        } else {
            last_clause.and(filter)
        };
        clauses.push(combined_clause);
    } else {
        clauses.push(filter);
    }
}

// Helper method to build ORDER BY clause
pub fn build_order_by_clause(order_by: &HashMap<String, OrderBy>) -> String {
    if order_by.is_empty() {
        return String::with_capacity(64);
    }

    let mut order_by_sql = String::with_capacity(64 * order_by.len());
    order_by_sql.push_str("ORDER BY ");

    for (i, (col, asc)) in order_by.into_iter().enumerate() {
        if i > 0 {
            order_by_sql.push_str(", ");
        }
        let direction = match asc {
            OrderBy::Asc => "ASC",
            OrderBy::Desc => "DESC",
        };
        order_by_sql.push_str(col);
        order_by_sql.push(' ');
        order_by_sql.push_str(direction);
    }

    order_by_sql
}

// Helper method to build LIMIT/OFFSET clause
pub fn build_limit_offset_clause<T: Debug + Clone>(
    limit: T,
    offset: Option<T>,
) -> (String, Vec<T>) {
    let mut limit_offset_sql = String::new();
    let mut values = Vec::new();

    limit_offset_sql.push_str("LIMIT ?");
    values.push(limit);

    if let Some(offset) = offset {
        limit_offset_sql.push_str(" OFFSET ?");
        values.push(offset);
    }

    (limit_offset_sql, values)
}

/// Builds a RETURNING clause for a SQL query.
pub fn build_returning_clause(columns: &[&str]) -> String {
    let mut returning_sql = String::with_capacity(80);
        returning_sql.push_str(" RETURNING ");
        for (i, col) in columns.iter().enumerate() {
            if i > 0 {
                returning_sql.push_str(", ");
            }
            returning_sql.push_str(col);
        }
        returning_sql
}
