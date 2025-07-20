use crate::common::builder::BuilderTrait;
#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
use crate::sql::filter::Expr;
use std::fmt::Debug;

use super::{cte::WithCTE, helper::build_returning_clause};

// INSERT-specific builder
#[derive(Default, Debug, Clone)]
pub struct InsertBuilder<T: Debug + Clone> {
    sql: String,
    values: Vec<T>,
    pos: Vec<usize>,
}

impl<T: Debug + Clone> InsertBuilder<T> {

    /// Specifies the table for the INSERT statement.
    /// 
    /// # Parameters
    /// - `table`: Name of the table to insert into.
    ///
    /// # Returns
    /// - `InsertBuilder`: Initialized InsertBuilder instance.
    pub fn into(table: &str) -> Self {
        let mut sql = String::with_capacity(table.len() + 12);
        sql.push_str("INSERT INTO ");
        sql.push_str(table);
        Self {
            sql,
            values: vec![],
            pos: vec![],
        }
    }

    /// Specifies the columns for the INSERT statement.
    /// 
    /// # Parameters
    /// - `columns`: List of column names to insert into.
    ///
    /// # Returns
    /// - `InsertBuilder`: Updated InsertBuilder instance.
    pub fn columns(mut self, columns: &[&str]) -> Self {
        self.sql.push_str(" (");
        for column in columns {
            if !self.sql.ends_with('(') {
                self.sql.push_str(", ");
            }
            self.sql.push_str(column);
        }
        self.sql.push_str(") ");
        self
    }

    /// Adds VALUES clause to the INSERT statement.
    /// 
    /// # Parameters
    /// - `values`: List of values to insert into the table.
    ///
    /// # Returns
    /// - `InsertBuilder`: Updated InsertBuilder instance.
    pub fn values(mut self, values: Vec<Vec<T>>) -> Self {
        self.sql.push_str("VALUES");
        let mut cols_values = Vec::new();
        for (row_idx, row) in values.into_iter().enumerate() {
            if row_idx > 0 {
                self.sql.push(',');
            }
            self.sql.push(' ');
            self.sql.push('(');
            for (i, val) in row.into_iter().enumerate() {
                if i > 0 {
                    self.sql.push_str(", ");
                }
                self.sql.push('?');
                self.pos.push(self.sql.len());
                cols_values.push(val);
            }
            self.sql.push(')');
        }

        if self.sql.ends_with(',') {
            self.sql.truncate(self.sql.len() - 1);
        }
        self.values = cols_values;
        self
    }

    /// Appends a new SQL query and parameter value to the existing query.
    pub fn append(mut self, sql: impl Into<String>, value: Vec<T>)-> Self {
        self.append_mut(sql, value);
        self
    }

    pub fn append_mut(&mut self, sql: impl Into<String>, value: Vec<T>)-> &mut Self {
        let sql = sql.into();
        
        self.sql.push_str(&sql);
        if !value.is_empty() {
            self.values.extend(value);
        }        
        self
    }

    /// Adds a WITH clause to the SELECT statement.
    /// Supported in Mysql 8.0+、Sqlite 3.8.3+ only.
    pub fn with(mut self, with_cte: WithCTE<T>) -> Self {
        let (with_sql, with_values) = with_cte.build();
        let mut new_sql = String::with_capacity(with_sql.len() + self.sql.len());
        new_sql.push_str(&with_sql);
        new_sql.push_str(&self.sql);
        self.sql = new_sql;
        self.values.extend(with_values);
        self
    }

    /// NOTE: Supported in Mysql 8.0.21+、Sqlite 3.35+ only.
    pub fn returning(mut self, columns: &[&str]) -> Self {
        self.sql.push_str(&build_returning_clause(columns));
        self
    }

    #[cfg(any(feature = "sqlite", feature = "postgres"))]
    /// Adds an `ON CONFLICT` clause with a `DO UPDATE` action.
    /// NOTE: Supported in Sqlite 3.24+ 、PostgreSQL only.
    pub fn on_conflict_do_update(mut self, conflict_target: &[&str], excluded_columns: &[&str], condition: Option<Expr<T>>) -> Self {
        let quote = |name: &&str| format!("\"{name}\"");
        let mut sql = String::with_capacity(80);

        sql.push_str(" ON CONFLICT (");
        sql.push_str(&conflict_target.iter().map(quote).collect::<Vec<_>>().join(", "));
        
        sql.push_str(") DO UPDATE SET ");
        
        for (i, &col) in excluded_columns.iter().enumerate() {
            if i > 0 { sql.push_str(", ") }
            sql.push_str(&format!("\"{col}\" = EXCLUDED.\"{col}\""));
        }
        self.append_mut(sql, vec![]);

        if let Some(expr) = condition {
            let mut where_cls = String::with_capacity(30);
            let (cond_sql, cond_values) = expr.build();
            where_cls.push_str(" WHERE ");
            where_cls.push_str(&cond_sql);
            self.append_mut(where_cls, cond_values);
        }
        self
    }

    /// NOTE: Mysql(`ON DUPLICATE`) only.
    #[cfg(feature = "mysql")]
    pub fn on_duplicate(mut self, excluded_columns: &[&str], condition: Option<Expr<T>>) -> Self {
        let quote = |name: &str| format!("`{}`", name);

        self.sql.push_str(" ON DUPLICATE KEY UPDATE ");
        for (i, &col) in excluded_columns.iter().enumerate() {
            if i > 0 {
                self.sql.push_str(", ");
            }

            let quoted_col = quote(col);
            self.sql.push_str(&quoted_col);
            self.sql.push_str(" = ");

            if let Some(ref expr) = condition {
                let (cond_sql, cond_values) = expr.clone().build();
                self.sql.push_str("IF(");
                self.sql.push_str(&cond_sql);
                self.sql.push_str(", VALUES(");
                self.sql.push_str(&quoted_col);
                self.sql.push_str("), ");
                self.sql.push_str(&quoted_col);
                self.sql.push_str(")");
                self.values.extend(cond_values);
            } else {
                self.sql.push_str("VALUES(");
                self.sql.push_str(&quoted_col);
                self.sql.push_str(")");
            }
        }

        self
    }

    /// Replaces an expression at a specific index in the SQL string.
    pub fn replace_expr_at(mut self, index: usize, expr_sql: impl Into<String>) -> Self {
        self.replace_expr_at_mut(index, expr_sql);
        self
    }

    /// Replaces an expression at a specific index in the SQL string, modifying self.
    pub fn replace_expr_at_mut(&mut self, index: usize, expr_sql: impl Into<String>) -> &mut Self {
        if index >= self.pos.len() {
            return self;
        }

        let expr = expr_sql.into();
        let replace_pos = match self.pos.get(index) {
            Some(&pos) => pos,
            None => return self,
        };

        self.sql.replace_range(replace_pos - 1..replace_pos, &expr);

        let delta = expr.len() - 1;
        for pos in &mut self.pos[index + 1..] {
            *pos += delta;
        }

        self.values.remove(index);
        self.pos.remove(index);

        self
    }
    
}

impl<T: Debug + Clone> BuilderTrait<T> for InsertBuilder<T> {
    /// Build method implementation for InsertBuilder, consuming self
    fn build(self) -> (String, Vec<T>) {
        (self.sql, self.values)
    }
}