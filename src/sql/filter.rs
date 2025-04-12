use std::fmt::Debug;

use crate::common::builder::BuilderTrait;

use super::select::SelectBuilder;

/// Filter query clause builder, used to create query conditions.
#[derive(Default, Debug, Clone)]
pub struct Expr<T: Debug + Clone> {
    /// Stores condition string.
    clause: String,
    /// Stores parameter values.
    values: Vec<T>,
}

impl<T: Debug + Clone> Expr<T> {
    /// Creates a new Filter builder with a specific operator.
    /// 
    /// # Parameters
    /// - `column`: Column name.
    /// - `op`: Operator.
    /// - `value`: Parameter value.
    ///
    /// # Returns
    /// - `Expr`: Initialized filter clause builder instance.
    pub fn new<U>(column: &str, op: &str, value: U) -> Self
    where
        U: Into<T>,
    {
        let mut clause = String::with_capacity(column.len() + op.len() + 3); // Estimated length
        clause.push_str(column);
        clause.push_str(" ");
        clause.push_str(op);
        clause.push_str(" ?");
        Expr {
            clause,
            values: vec![value.into()],
        }
    }

    /// Creates an Exprression query condition without binding any parameter values.
    pub fn from_str(expr: impl Into<String>) -> Self {
        Expr { clause: expr.into(), values: vec![] }
    }

    /// Gets the Filter clause string.
    ///
    /// # Returns
    /// - `(String, Vec<T>)`: Filter clause string and parameter values list.
    pub fn build(self) -> (String, Vec<T>) {
        (self.clause, self.values)
    }

    fn and_or(&mut self, other: Expr<T>, op: &str) -> &mut Self {
        let mut new_clause = String::with_capacity(self.clause.len() + other.clause.len() + 5);
        new_clause.push_str(&self.clause);
        new_clause.push_str(op);
        new_clause.push_str(&other.clause);
        self.clause = new_clause;
        self.values.extend(other.values);
        self
    }

    /// Combines multiple Expr using AND connection.
    /// 
    /// # Returns
    /// - `Expr<T>`: A new Expr instance with the combined conditions.
    pub fn and(mut self, other: Expr<T>) -> Self {
        self.and_or(other, " AND ");
        self
    }
    
    
    /// Combines multiple Expr using OR connection.
    /// 
    /// # Returns
    /// - `Expr<T>`: A new Expr instance with the combined conditions.
    pub fn or(mut self, other: Expr<T>) -> Self {
        self.and_or(other, " OR ");
        self
    }

    fn add_subquery(subquery: SelectBuilder<T>, op: &str) -> (String, Vec<T>) {
        let (subquery_sql, subquery_values) = subquery.build();
        let mut newsql = String::with_capacity(subquery_sql.len() + 12);
        newsql.push_str(op);
        newsql.push_str("(");
        newsql.push_str(&subquery_sql);
        newsql.push_str(")");
        (newsql, subquery_values)
    }

    /// Creates an IN subquery condition.
    ///
    /// # Arguments
    /// * `subquery` - The subquery to be used in the IN condition.
    ///
    /// # Returns
    /// - `Expr<T>`: A new Expr instance with the IN subquery condition.
    pub fn in_subquery(column: &str, subquery: SelectBuilder<T>) -> Self {
        let mut clause = String::with_capacity(60);
        let (query_sql, values) = Self::add_subquery(subquery, " IN ");
        clause.push_str(column);
        clause.push_str(&query_sql);
        Expr { clause, values }
    }

    /// Creates an EXISTS subquery condition.
    ///
    /// # Arguments
    /// * `subquery` - The subquery to be used in the EXISTS condition.
    ///
    /// # Returns
    /// - `Expr<T>`: A new Expr instance with the EXISTS subquery condition.
    pub fn exists(subquery: SelectBuilder<T>) -> Self {
        let (clause, values) = Self::add_subquery(subquery, " EXISTS ");
        Expr { clause, values }
    }

    /// Creates a NOT EXISTS subquery condition.
    ///
    /// # Arguments
    /// * `subquery` - The subquery to be used in the NOT EXISTS condition.
    ///
    /// # Returns
    /// - `Expr<T>`: A new Expr instance with the NOT EXISTS subquery condition.
    pub fn not_exists(subquery: SelectBuilder<T>) -> Self {
        let (clause, values) = Self::add_subquery(subquery, " NOT EXISTS ");
        Expr { clause, values }
    }

    /// Creates a new Expr with a specific column name.
    /// 
    /// # Parameters
    /// - `column`: Column name.
    ///
    /// # Returns
    /// - `ColumnExpr`: Initialized filter clause builder instance.
    pub fn col<'a>(column: &'a str) -> ColumnExpr<T> {
        ColumnExpr { inner: Self::from_str(column)}
    }

}

/// Simplifies writing, creates a Expr for field value comparison query.
pub struct ColumnExpr<T: Debug + Clone> {
    inner: Expr<T>,
}

impl<T: Debug + Clone> ColumnExpr<T> {

    fn with(mut self, op: &str, value: impl Into<T>) -> Expr<T> {
        self.inner.clause.push_str(" ");
        self.inner.clause.push_str(op);
        self.inner.clause.push_str(" ?");
        self.inner.values.push(value.into());
        self.inner
    }

    /// Creates an equal condition.
    ///
    /// # Parameters
    /// - `value`: Parameter value.
    ///
    /// # Returns
    /// - `Expr`: Initialized filter clause builder instance.
    pub fn eq(self, value: impl Into<T>) -> Expr<T> 
    {
        self.with("=", value)
    }

    /// Creates a greater than condition.
    ///
    /// # Parameters
    /// - `value`: Parameter value.
    ///
    /// # Returns
    /// - `Expr`: Initialized filter clause builder instance.
    pub fn gt(self, value: impl Into<T>) -> Expr<T> {
        self.with(">", value)
    }

    /// Creates a less than condition.
    ///
    /// # Parameters
    /// - `value`: Parameter value.
    ///
    /// # Returns
    /// - `Expr`: Initialized filter clause builder instance.
    pub fn lt(self, value: impl Into<T>) -> Expr<T> {
        self.with("<", value)
    }

    /// Creates a greater than or equal condition.
    ///
    /// # Parameters
    /// - `value`: Parameter value.
    ///
    /// # Returns
    /// - `Expr`: Initialized filter clause builder instance.
    pub fn gte(self, value: impl Into<T>) -> Expr<T> {
        self.with(">=", value)
    }

    /// Creates a less than or equal condition.
    ///
    /// # Parameters
    /// - `value`: Parameter value.
    ///
    /// # Returns
    /// - `Expr`: Initialized filter clause builder instance.
    pub fn lte(self, value: impl Into<T>) -> Expr<T> {
        self.with("<=", value)
    }

    /// Creates a LIKE condition.
    ///
    /// # Parameters
    /// - `value`: Parameter value.
    ///
    /// # Returns
    /// - `Expr`: Initialized filter clause builder instance.
    pub fn like(self, value: impl Into<T>) -> Expr<T> {
        self.with("LIKE", value)
    }

    /// Creates a not equal condition.
    ///
    /// # Parameters
    /// - `value`: Parameter value.
    ///
    /// # Returns
    /// - `Expr`: Initialized filter clause builder instance.
    pub fn ne(self, value: impl Into<T>) -> Expr<T> {
        self.with("!=", value)
    }
    
    /// Creates an IS NULL or IS NOT NULL query condition.
    fn null_or_not(mut self, not: bool) -> Expr<T> {
        let operator = if not { "IS NOT NULL" } else { "IS NULL" };
        self.inner.clause.push_str(" ");
        self.inner.clause.push_str(operator);
        self.inner
    }

    /// Creates an IS NULL condition.
    ///
    /// # Returns
    /// - `Expr`: Initialized filter clause builder instance.
    pub fn is_null(self) -> Expr<T> {
        self.null_or_not(false)
    }

    /// Creates an IS NOT NULL condition.
    ///
    /// # Returns
    /// - `Expr`: Initialized filter clause builder instance.
    pub fn is_not_null(self) -> Expr<T> {
        self.null_or_not(true)
    }

    /// Creates an IN or NOT IN query condition.
    fn in_or_not_in<I, U>(mut self, values: I, not: bool) -> Expr<T>
    where
        I: IntoIterator<Item = U>,
        U: Into<T>,
    {
        let converted_values: Vec<T> = values.into_iter().map(|v| v.into()).collect();
        let placeholders = vec!["?"; converted_values.len()].join(", ");
        let operator = if not { "NOT IN" } else { "IN" };
        self.inner.clause.push_str(" ");
        self.inner.clause.push_str(operator);
        self.inner.clause.push_str(" (");
        self.inner.clause.push_str(&placeholders);
        self.inner.clause.push_str(")");
        self.inner.values = converted_values;
        self.inner
    }

    /// Creates an IN condition.
    ///
    /// # Parameters
    /// - `values`: Parameter values list.
    ///
    /// # Returns
    /// - `Expr`: Initialized filter clause builder instance.
    pub fn in_<I, U>(self, values: I) -> Expr<T>
    where
        I: IntoIterator<Item = U>,
        U: Into<T>,
    {
        self.in_or_not_in(values, false)
    }

    /// Creates a NOT IN condition.
    ///
    /// # Parameters
    /// - `values`: Parameter values list.
    ///
    /// # Returns
    /// - `Expr`: Initialized filter clause builder instance.
    pub fn not_in<I, U>(self, values: I) -> Expr<T>
    where
        I: IntoIterator<Item = U>,
        U: Into<T>,
    {
        self.in_or_not_in(values, true)
    }

    /// Creates a BETWEEN condition.
    ///
    /// # Parameters
    /// - `value1`: First parameter value.
    /// - `value2`: Second parameter value.
    ///
    /// # Returns
    /// - `Expr`: Initialized filter clause builder instance.
    pub fn between(mut self, value1: impl Into<T>, value2: impl Into<T>) -> Expr<T> {
        self.inner.clause.push_str(" ");
        self.inner.clause.push_str(" BETWEEN ? AND ?");
        self.inner.values.push(value1.into());
        self.inner.values.push(value2.into());
        self.inner
    }
}