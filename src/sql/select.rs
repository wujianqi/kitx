use std::fmt::Debug;
use crate::common::builder::{BuilderTrait, QueryTrait, FilterTrait};
use super::{
    agg::Agg, case_when::CW, cte::WithCTE, filter::Expr, helper::{
        build_limit_offset_clause, 
        build_order_by_clause, 
        build_where_clause, 
        combine_where_clause
    }, join::Join
};

// SELECT-specific builder
#[derive(Default, Debug, Clone)]
pub struct SelectBuilder<T: Debug + Clone> {
    sql: String,
    values: Vec<T>,
    where_clauses: Vec<Expr<T>>,
    order_by_clauses: Vec<(String, bool)>,
    limit_offset: Option<(T, Option<T>)>,
    joins: Vec<Join<T>>,
    group_having: Option<(String, Vec<T>)>,
}

impl<T: Debug + Clone> SelectBuilder<T> {
    /// Specifies the columns for the SELECT statement.
    /// 
    /// # Parameters
    /// - `columns`: Array of column names.
    ///
    /// # Returns
    /// - `SelectBuilder`: Initialized SelectBuilder instance.
    pub fn columns(columns: &[&str]) -> Self {
        let mut sql = String::with_capacity(100);
        sql.push_str("SELECT ");
        if columns.is_empty() {
            sql.push_str("*");
        }
        for (i, col) in columns.iter().enumerate() {
            if i > 0 {
                sql.push_str(", ");
            }
            sql.push_str(col);
        }
        Self {
            sql,
            values: vec![],
            where_clauses: vec![],
            order_by_clauses: vec![],
            limit_offset: None,
            joins: vec![],
            group_having: None,
        }
    }

    /// Specifies the table for the SELECT statement.
    /// 
    /// # Parameters
    /// - `table`: Table name.
    ///
    /// # Returns
    /// - `SelectBuilder`: Initialized SelectBuilder instance.
    pub fn from(mut self, table: &str) -> Self {
        self.sql.push_str(" FROM ");
        self.sql.push_str(table);
        self
    }

    /// Adds a WHERE clause to the SELECT statement.
    /// 
    /// # Parameters
    /// - `filter`: WHERE clause.
    ///
    /// # Returns
    /// - `SelectBuilder`: Updated SelectBuilder instance.
    pub fn where_(mut self, filter: Expr<T>) -> Self {
        self.where_mut(filter);
        self
    }
    

    /// Adds an AND condition to the last WHERE clause.
    /// 
    /// # Parameters
    /// - `filter`: WHERE clause.
    ///
    /// # Returns
    /// - `SelectBuilder`: Updated SelectBuilder instance.
    pub fn and(mut self, filter: Expr<T>) -> Self {
        self.and_mut(filter);
        self
    }

    /// Adds an OR condition to the last WHERE clause.
    /// 
    /// # Parameters
    /// - `filter`: WHERE clause.
    ///
    /// # Returns
    /// - `SelectBuilder`: Updated SelectBuilder instance.
    pub fn or(mut self, filter: Expr<T>) -> Self {
        self.or_mut(filter);
        self
    }

    /// Adds a JOIN clause to the SELECT statement.
    /// 
    /// # Parameters
    /// - `join`: Join clause.
    ///
    /// # Returns
    /// - `SelectBuilder`: Updated SelectBuilder instance.
    pub fn join(mut self, join: Join<T>) -> Self {
        self.joins.push(join);
        self
    }

    /// Adds an aggregate function to the SELECT statement.
    /// Only for aggregated groups.
    pub fn aggregate(mut self, agg: Agg<T>) -> Self {
        self.sql.push_str(&agg.build_aggregates());
        self.group_having = agg.build_group_having();
        self
    }
    
    /// Adds a CASE WHEN clause to the SELECT statement.
    pub fn case_when(mut self, case_when: CW<T>) -> Self {
        let (case_when_sql, case_when_values) = case_when.build();
        self.sql.push_str(", ");
        self.sql.push_str(&case_when_sql);
        self.values.extend(case_when_values);
        self
    }

    /// Adds an ORDER BY clause to the SELECT statement.
    /// 
    /// # Parameters
    /// - `column`: Column name.
    /// - `ascending`: Whether to sort in ascending order.
    ///
    /// # Returns
    /// - `SelectBuilder`: Updated SelectBuilder instance.
    pub fn order_by(mut self, column: &str, ascending: bool) -> Self {
        self.order_by_mut(column, ascending);
        self
    }

    /// Adds a LIMIT/OFFSET clause to the SELECT statement.
    /// 
    /// # Parameters
    /// - `limit`: Limit value.
    /// - `offset`: Offset value.
    ///
    /// # Returns
    /// - `SelectBuilder`: Updated SelectBuilder instance.
    pub fn limit_offset(mut self, limit: impl Into<T>, offset: Option<impl Into<T>>) -> Self {
        self.limit_offset_mut(limit, offset);
        self
    }

    /// Adds a subquery to the SELECT statement.
    pub fn subquery(mut self, subquery: SelectBuilder<T>, alias: Option<&str>) -> Self {
        let (subquery_sql, subquery_values) = subquery.build();
        self.sql.push_str(" (");
        self.sql.push_str(&subquery_sql);
        self.sql.push_str(")");
        if let Some(alias) = alias {
            self.sql.push_str(" AS ");
            self.sql.push_str(alias);
        }
        self.values.extend(subquery_values);
        self
    }

    /// Adds a UNION clause to the SELECT statement.
    pub fn union(mut self, other: SelectBuilder<T>, all: bool) -> Self {
        let (other_sql, other_values) = other.build();
        let union_keyword = if all { "UNION ALL" } else { "UNION" };

        // Append the UNION clause and the other SQL query
        self.sql.push_str(" ");
        self.sql.push_str(union_keyword);
        self.sql.push_str(" ");
        self.sql.push_str(&other_sql);

        // Merge parameter values
        self.values.extend(other_values);

        self
    }

    
    /// Creates a new SelectBuilder instance with the given SQL query and parameter values.
    pub fn raw(sql: impl Into<String>, params: Option<Vec<T>>) -> Self {
        let sql = sql.into();
        let mut values = vec![];
        if let Some(vals) = params {
            values.extend(vals);
        }
        Self {
            sql,
            values,
            where_clauses: vec![],
            order_by_clauses: vec![],
            limit_offset: None,
            group_having: None,
            joins: vec![],
        }        
    }

    /// Appends a new SQL query and parameter value to the existing query.
    pub fn append(mut self, sql: impl Into<String>, value: Option<T>)-> Self {
        let sql = sql.into();
        let mut values = vec![];
        if let Some(val) = value {
            values.push(val);
        }
        self.sql.push_str(&sql);
        self.values.extend(values);
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

}


impl<T: Debug + Clone> FilterTrait<T> for SelectBuilder<T> {
    type Expr = Expr<T>;    
    /// Adds a WHERE clause to the SELECT statement.
    fn where_mut(&mut self, filter: Expr<T>) -> &mut Self {
        self.where_clauses.push(filter);
        self
    }

    /// Adds an AND condition to the last WHERE clause.
    fn and_mut(&mut self, filter: Expr<T>) -> &mut Self {
        combine_where_clause(&mut self.where_clauses, filter, false);
        self
    }

    /// Adds an OR condition to the last WHERE clause.
    fn or_mut(&mut self, filter: Expr<T>) -> &mut Self {
        combine_where_clause(&mut self.where_clauses, filter, true);
        self
    }
    
}


impl<T: Debug + Clone> QueryTrait<T> for SelectBuilder<T> {
    /// Adds an ORDER BY clause to the SELECT statement.
    fn order_by_mut(&mut self, column: &str, ascending: bool) -> &mut Self {
        if let Some(index) = self.order_by_clauses.iter().position(|(col, _)| col == column) {
            // Replace the existing order by clause
            self.order_by_clauses[index] = (column.to_string(), ascending);
        } else {
            // Add a new order by clause
            self.order_by_clauses.push((column.to_string(), ascending));
        }
        self
    }

    /// Adds a LIMIT/OFFSET clause to the SELECT statement.
    fn limit_offset_mut(&mut self, limit: impl Into<T>, offset: Option<impl Into<T>>) -> &mut Self {
        self.limit_offset = Some((limit.into(), offset.map(|o| o.into())));
        self
    }

}

impl<T: Debug + Clone> BuilderTrait<T> for SelectBuilder<T> {
    fn build(self) -> (String, Vec<T>) {
        let mut final_sql = self.sql;
        let mut values = self.values;

        // Process JOIN clauses
        let mut is_first_join = true;
        for join in self.joins {
            let (join_sql, join_values) = join.build();
            if !is_first_join {
                final_sql.push(' ');
            }
            final_sql.push_str(&join_sql);
            values.extend(join_values);
            is_first_join = false;
        }

        // Process WHERE clauses
        if !self.where_clauses.is_empty() {
            let where_clauses = &self.where_clauses;
            let (where_sql, where_values) = build_where_clause(where_clauses.to_vec());
            final_sql.push_str(" ");
            final_sql.push_str(&where_sql);
            values.extend(where_values);
        }

        // Process GROUP BY and HAVING clauses
        if let Some((group_having_sql, group_having_values)) = &self.group_having {
            final_sql.push_str(&group_having_sql);
            values.extend(group_having_values.to_vec());
        }

        // Process ORDER BY clauses
        if !self.order_by_clauses.is_empty() {
            let order_by_clauses = &self.order_by_clauses;
            let order_by_sql = build_order_by_clause(order_by_clauses.to_vec());
            final_sql.push_str(" ");
            final_sql.push_str(&order_by_sql);
        }

        // Process LIMIT/OFFSET clauses
        if let Some((limit, offset)) = self.limit_offset {
            let (limit_offset_sql, limit_offset_values) = build_limit_offset_clause(limit, offset);
            final_sql.push(' ');
            final_sql.push_str(&limit_offset_sql);
            values.extend(limit_offset_values);
        }

        (final_sql, values)
    }
}