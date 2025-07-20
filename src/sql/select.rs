use std::{borrow::Cow, collections::HashMap, fmt::Debug, mem::take};
use crate::common::{builder::{BuilderTrait, FilterTrait}, types::OrderBy};
use super::{
    agg::Func, case_when::CaseWhen, cte::WithCTE, filter::Expr, helper::{
        build_limit_offset_clause, 
        build_order_by_clause, 
        build_where_clause, 
        combine_where_clause
    }, join::JoinType
};

// SELECT-specific builder
#[derive(Debug, Clone, Default)]
pub struct SelectBuilder<T: Debug + Clone> {
    sql: String,
    columns: Vec<String>,
    values: Vec<T>,
    where_clauses: Vec<Expr<T>>,
    order_by_clauses: HashMap<String, OrderBy>,
    limit_offset: Option<(T, Option<T>)>,
    joins: Vec<JoinType<T>>,
    group_having: Option<(String, Vec<T>)>,
    table_name: String,
    alias_name: Option<String>,
    is_distinct: bool,
}

impl<T: Debug + Clone + Default> SelectBuilder<T> {
    /// Creates an empty SELECT statement.
    pub fn empty_columns() -> Self {
        Self {
            sql: String::from("SELECT "),
            columns: Vec::new(),
            is_distinct: false,
            ..Default::default()
        }
    }

    /// Specifies the columns for the SELECT statement.
    /// 
    /// # Parameters
    /// - `columns`: Array of column names.
    ///
    /// # Returns
    /// - `SelectBuilder`: Initialized SelectBuilder instance.
    pub fn columns(columns: &[&str]) -> Self {
        Self {
            sql: String::from("SELECT "),
            columns: columns.iter().map(|s| s.to_string()).collect(),
            is_distinct: false,
            ..Default::default()
        }
    }

    /// Specifies whether to use DISTINCT in the SELECT statement.
    pub fn distinct(mut self) -> Self {
        self.is_distinct = true;
        self
    }

    /// Specifies the table for the SELECT statement.
    /// 
    /// # Parameters
    /// - `table`: Table name.
    ///
    /// # Returns
    /// - `SelectBuilder`: Initialized SelectBuilder instance.
    pub fn from(mut self, table: &str) -> Self {
        self.table_name = table.to_string();
        self
    }

    /// Specifies the alias for the table.
    pub fn alias(mut self, alias: &str) -> Self {
        self.alias_mut(alias);
        self
    }

    /// Specifies the alias for the table.
    pub fn alias_mut(&mut self, alias: &str) -> &mut Self {
        if alias.is_empty() {
            return self;
        }        
        self.alias_name = Some(alias.to_string());

        fn rewrite_col<'a>(col: &'a str, alias: &str, table: &str) -> Cow<'a, str> {
            if let Some((prefix, suffix)) = col.split_once('.') {
                if prefix == table {
                    return Cow::Owned(format!("{}.{}", alias, suffix));
                }
            } else if !col.contains('(') && !col.contains(' ') {
                return Cow::Owned(format!("{}.{}", alias, col));
            }
            Cow::Borrowed(col)
        }

        let new_columns: Vec<String> = self.columns.iter()
            .map(|col| rewrite_col(col, alias, &self.table_name).into_owned())
            .collect();

        self.columns = new_columns;
        self
    }
    

    /// Adds an AND condition to the last WHERE clause.
    /// 
    /// # Parameters
    /// - `filter`: WHERE clause.
    ///
    /// # Returns
    /// - `SelectBuilder`: Updated SelectBuilder instance.
    pub fn and_where(mut self, filter: Expr<T>) -> Self {
        self.and_where_mut(filter);
        self
    }

    /// Adds an OR condition to the last WHERE clause.
    /// 
    /// # Parameters
    /// - `filter`: WHERE clause.
    ///
    /// # Returns
    /// - `SelectBuilder`: Updated SelectBuilder instance.
    pub fn or_where(mut self, filter: Expr<T>) -> Self {
        self.or_where_mut(filter);
        self
    }

    /// Adds a JOIN clause to the SELECT statement.
    /// 
    /// # Parameters
    /// - `join`: Join clause.
    ///
    /// # Returns
    /// - `SelectBuilder`: Updated SelectBuilder instance.
    pub fn join(mut self, join_clauses: JoinType<T>) -> Self {
        self.join_mut(join_clauses);
        self
    }

    pub fn join_mut(&mut self, join_clauses: JoinType<T>) -> &mut Self {
        if self.alias_name.is_none() {
            let table_name = self.table_name.clone();
            self.alias_mut(&table_name);
        }
        self.joins.push(join_clauses);
        self
    }

    /// Adds an aggregate function to the SELECT statement.
    /// Only for aggregated groups.
    pub fn aggregate(mut self, agg: Func<T>) -> Self {
        self.aggregate_mut(agg);
        self
    }

    pub fn aggregate_mut(&mut self, agg: Func<T>) -> &mut Self {
        if !self.sql.ends_with("SELECT ") {
            self.sql.push_str(", ");
        }
        self.sql.push_str(&agg.build_aggregates());
        self.group_having = agg.build_group_having();
        self
    }
    
    /// Adds a CASE WHEN clause to the SELECT statement.
    pub fn case_when(mut self, case_when: CaseWhen<T>) -> Self {
        self.case_when_mut(case_when);
        self
    }

    pub fn case_when_mut(&mut self, case_when: CaseWhen<T>) -> &mut Self {
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
    pub fn order_by(mut self, column: &str, ordering: OrderBy) -> Self {
        self.order_by_mut(column, ordering);
        self
    }
    
    /// Adds an ORDER BY clause to the SELECT statement.
    pub fn order_by_mut(&mut self, column: &str, ordering: OrderBy) -> &mut Self {
        self.order_by_clauses.insert(column.to_string(), ordering);
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
    
    /// Adds a LIMIT/OFFSET clause to the SELECT statement.
    pub fn limit_offset_mut(&mut self, limit: impl Into<T>, offset: Option<impl Into<T>>) -> &mut Self {
        self.limit_offset = Some((limit.into(), offset.map(|o| o.into())));
        self
    }

    /// Adds a subquery to the SELECT statement.
    pub fn subquery(mut self, subquery: SelectBuilder<T>, alias: Option<&str>) -> Self {        
        self.subquery_mut(subquery, alias);
        self
    }

    /// Adds a subquery to the SELECT statement.
    pub fn subquery_mut(&mut self, subquery: SelectBuilder<T>, alias: Option<&str>) -> &mut Self {
        let (subquery_sql, subquery_values) = subquery.build();
        let alias_len = alias.map(|a| a.len() + 4).unwrap_or(0);
        self.sql.reserve(subquery_sql.len() + alias_len + 3);
        
        self.sql.push_str(" (");
        self.sql.push_str(&subquery_sql);
        self.sql.push(')');
        
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
        self.sql.reserve(union_keyword.len() + other_sql.len());
        
        self.sql.push(' ');
        self.sql.push_str(union_keyword);
        self.sql.push_str(&other_sql);        
        self.values.extend(other_values);
        self
    }

    /// Appends a new SQL query and parameter value to the existing query.
    pub fn append(mut self, sql: impl Into<String>, value: Vec<T>)-> Self {
        self.append_mut(sql, value);
        self
    }

    /// Appends a new SQL query and parameter value to the existing query.
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
        let new_capacity = with_sql.len() + self.sql.len() + 1;
        let mut new_sql = String::with_capacity(new_capacity);
        
        new_sql.push_str(&with_sql);
        new_sql.push(' ');
        new_sql.push_str(&self.sql);
        
        self.sql = new_sql;
        self.values.extend(with_values);
        self
    }

    /// Returns a reference to the WHERE clauses.
    pub fn take_where_clauses(self) -> Vec<Expr<T>> {
        self.where_clauses
    }

}


impl<T: Debug + Clone> FilterTrait<T> for SelectBuilder<T> {
    type Expr = Expr<T>;
    /// Adds an AND condition to the last WHERE clause.
    fn and_where_mut<F>(&mut self, filter: F) -> &mut Self
    where
        F: Into<Self::Expr>
    {
        combine_where_clause(&mut self.where_clauses, filter.into(), false);
        self
    }

    /// Adds an OR condition to the last WHERE clause.
    fn or_where_mut<F>(&mut self, filter: F) -> &mut Self
    where
        F: Into<Self::Expr>
    {
        combine_where_clause(&mut self.where_clauses, filter.into(), true);
        self
    }
}

impl<T: Debug + Clone> BuilderTrait<T> for SelectBuilder<T> {
    fn build(mut self) -> (String, Vec<T>) {
        let mut values = take(&mut self.values);
        let mut sql = String::from("SELECT ");

        if self.is_distinct {
            sql.push_str("DISTINCT ");
        }

        if !self.columns.is_empty() {
            sql.push_str(&self.columns.join(", "));
        } else {
            sql.push('*');
        }

        if !self.table_name.is_empty() {
            sql.push_str(" FROM ");
            sql.push_str(&self.table_name);

            if let Some(ref alias) = self.alias_name {
                sql.push_str(" AS ");
                sql.push_str(alias);
            }
        }

        for join in self.joins.drain(..) {
            let (join_sql, join_values) = join.build();
            sql.push(' ');
            sql.push_str(&join_sql);
            values.extend(join_values);
        }

        if !self.where_clauses.is_empty() {
            let (where_sql, where_values) = build_where_clause(self.where_clauses);
            sql.push(' ');
            sql.push_str(&where_sql);
            values.extend(where_values);
        }

        if let Some((group_having_sql, group_having_values)) = self.group_having {
            sql.push_str(&group_having_sql);
            values.extend(group_having_values);
        }

        if !self.order_by_clauses.is_empty() {
            let order_by_sql = build_order_by_clause(&self.order_by_clauses);
            sql.push_str(" ");
            sql.push_str(&order_by_sql);
        }

        if let Some((limit, offset)) = self.limit_offset {
            let (limit_offset_sql, limit_offset_values) = build_limit_offset_clause(limit, offset);
            sql.push(' ');
            sql.push_str(&limit_offset_sql);
            values.extend(limit_offset_values);
        }

        (sql, values)
    }
}