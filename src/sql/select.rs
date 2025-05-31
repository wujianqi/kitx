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
#[derive(Debug, Clone)]
pub struct SelectBuilder<T: Debug + Clone> {
    sql: String,
    columns: Vec<String>,
    values: Vec<T>,
    where_clauses: Vec<Expr<T>>,
    order_by_clauses: HashMap<String, OrderBy>,
    limit_offset: Option<(T, Option<T>)>,
    joins: Vec<JoinType<T>>,
    group_having: Option<(String, Vec<T>)>,
    table_info: (String, bool),
    has_alias: bool,
}

impl<T: Debug + Clone> Default for SelectBuilder<T> {
    fn default() -> Self {
        Self {
            sql: String::with_capacity(256),
            columns: Vec::new(),
            values: Vec::new(),
            where_clauses: Vec::new(),
            order_by_clauses: HashMap::new(),
            limit_offset: None,
            joins: Vec::new(),
            group_having: None,
            table_info: (String::new(), false),
            has_alias: false,
        }
    }
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
        let mut capacity = 7;
        let mut cols = Vec::with_capacity(capacity);
        for col in columns {
            capacity += col.len() + 2;
            cols.push(col.to_string());
        }
        
        let mut sql = String::with_capacity(capacity);
        sql.push_str("SELECT ");
        
        for (i, col) in columns.iter().enumerate() {
            if i > 0 {
                sql.push_str(", ");
            }
            sql.push_str(col);
        }
        
        Self {
            sql,
            columns: cols,
            ..Default::default()
        }
    }

    /// Specifies whether to use DISTINCT in the SELECT statement.
    pub fn distinct(mut self) -> Self {
        if self.sql.starts_with("SELECT ") {
            self.sql = self.sql.replacen("SELECT ", "SELECT DISTINCT ", 1);
        }
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
        if self.columns.is_empty() {
            self.sql.push_str(" SELECT *");
            self.table_info.1 = true;
            self.columns = vec!["*".to_string()];
        }    
        
        let additional_capacity = 6 + table.len();
        self.sql.reserve(additional_capacity);
        
        self.sql.push_str(" FROM ");
        self.sql.push_str(table);
        self.table_info.0 = table.to_string();
        self
    }

    /// Specifies the alias for the table.
    pub fn alias(mut self, alias: &str) -> Self {
        self.alias_mut(alias);
        self
    }

    pub fn alias_mut(&mut self, alias: &str) -> &mut Self {
        if self.has_alias {
            return self;
        }
    
        let new_capacity = self.sql.len() + alias.len() * 2 + 10;
        let mut new_sql = String::with_capacity(new_capacity);
    
        new_sql.push_str("SELECT ");
    
        if self.table_info.1 {
            new_sql.push_str(alias);
            new_sql.push_str(".*");
        } else {
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

            let rewritten_cols: Vec<Cow<'_, str>> = self.columns.iter()
                .map(|col| rewrite_col(col, alias, &self.table_info.0))
                .collect();
    
            let temp: Vec<&str> = rewritten_cols.iter().map(|cow| cow.as_ref()).collect();
            new_sql.push_str(&temp.join(", "));
        }
    
        new_sql.push_str(" FROM ");
        new_sql.push_str(&self.table_info.0);
        new_sql.push_str(" AS ");
        new_sql.push_str(alias);
    
        if let Some(from_pos) = self.sql.find(" FROM ") {
            let from_end_pos = from_pos + " FROM ".len() + self.table_info.0.len();
            if let Some(rest) = self.sql.get(from_end_pos..) {
                let trimmed = rest.trim_start();
                if !trimmed.is_empty() {
                    if !new_sql.ends_with(|c: char| c.is_whitespace()) {
                        new_sql.push(' ');
                    }
                    new_sql.push_str(trimmed);
                }
            }
        }
    
        self.sql = new_sql;
        self.has_alias = true;
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
        if !self.has_alias {
            let table_name = take(&mut self.table_info.0);
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
    pub fn append(mut self, sql: impl Into<String>, value: Option<T>)-> Self {
        self.append_mut(sql, value);
        self
    }

    pub fn append_mut(&mut self, sql: impl Into<String>, value: Option<T>)-> &mut Self {
        let sql_str = sql.into();
        self.sql.reserve(sql_str.len());
        self.sql.push_str(&sql_str);
        
        if let Some(val) = value {
            self.values.push(val);
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
        let mut sql = take(&mut self.sql);

        // Process JOIN clauses
        for join in self.joins.drain(..) {
            let (join_sql, join_values) = join.build();
            sql.push(' ');
            sql.push_str(&join_sql);
            values.extend(join_values);
        }

        // Process WHERE clauses
        if !self.where_clauses.is_empty() {
            let (where_sql, where_values) = build_where_clause(self.where_clauses);
            sql.push(' ');
            sql.push_str(&where_sql);
            values.extend(where_values);
        }

        // Process GROUP BY and HAVING clauses
        if let Some((group_having_sql, group_having_values)) = self.group_having {
            sql.push_str(&group_having_sql);
            values.extend(group_having_values);
        }

        // Process ORDER BY clauses
        if !self.order_by_clauses.is_empty() {
            let order_by_sql = build_order_by_clause(&self.order_by_clauses);
            sql.push_str(" ");
            sql.push_str(&order_by_sql);
        }

        // Process LIMIT/OFFSET clauses
        if let Some((limit, offset)) = self.limit_offset {
            let (limit_offset_sql, limit_offset_values) = build_limit_offset_clause(limit, offset);
            sql.push(' ');
            sql.push_str(&limit_offset_sql);
            values.extend(limit_offset_values);
        }

        (sql, values)
    }
}