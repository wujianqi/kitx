use std::{fmt::Debug, marker::PhantomData};

/// Filter query clause builder, used to create query conditions.
#[derive(Default, Debug, Clone)]
pub struct FilterClause<T: Debug + Clone> {
    /// Stores condition string.
    clause: String,
    /// Stores parameter values.
    values: Vec<T>,
}

impl<T: Debug + Clone> FilterClause<T> {
    /// Creates a new Filter builder with a specific operator.
    pub fn new<U>(column: &str, op: &str, value: U) -> Self
    where
        U: Into<T>,
    {
        let mut clause = String::with_capacity(column.len() + op.len() + 3); // Estimated length
        clause.push_str(column);
        clause.push_str(" ");
        clause.push_str(op);
        clause.push_str(" ?");
        FilterClause {
            clause,
            values: vec![value.into()],
        }
    }

    /// Creates an expression query condition.
    pub fn expr(expr: impl Into<String>) -> FilterClause<T> {
        FilterClause { clause: expr.into(), values: vec![] }
    }

    /// Creates an IS NULL or IS NOT NULL query condition.
    fn null_or_not(column: &str, not: bool) -> Self {
        let operator = if not { "IS NOT NULL" } else { "IS NULL" };
        let mut clause = String::with_capacity(column.len() + operator.len() + 1);
        clause.push_str(column);
        clause.push_str(" ");
        clause.push_str(operator);
        FilterClause {
            clause,
            values: Vec::new(),
        }
    }

    /// Creates an IN or NOT IN query condition.
    fn in_or_not_in<I, U>(column: &str, values: I, not: bool) -> Self
    where
        I: IntoIterator<Item = U>,
        U: Into<T>,
    {
        let converted_values: Vec<T> = values.into_iter().map(|v| v.into()).collect();
        let placeholders = vec!["?"; converted_values.len()].join(", ");
        let operator = if not { "NOT IN" } else { "IN" };
        let mut clause = String::with_capacity(column.len() + operator.len() + placeholders.len() + 4);
        clause.push_str(column);
        clause.push_str(" ");
        clause.push_str(operator);
        clause.push_str(" (");
        clause.push_str(&placeholders);
        clause.push_str(")");
        FilterClause {
            clause,
            values: converted_values,
        }
    }

    /// Creates a BETWEEN query condition.
    fn between<U, V>(column: &str, value1: U, value2: V) -> Self
    where
        U: Into<T>,
        V: Into<T>,
    {
        let mut clause = String::with_capacity(column.len() + 13); // "BETWEEN ? AND ?" length is 13
        clause.push_str(column);
        clause.push_str(" BETWEEN ? AND ?");
        FilterClause {
            clause,
            values: vec![value1.into(), value2.into()],
        }
    }

    /// Gets the Filter clause string.
    ///
    /// # Returns
    /// - `(String, Vec<T>)`: Filter clause string and parameter values list.
    pub fn build(self) -> (String, Vec<T>) {
        (self.clause, self.values)
    }

    /// Combines multiple FilterClause using AND connection.
    pub fn and(mut self, other: FilterClause<T>) -> Self {
        let mut new_clause = String::with_capacity(self.clause.len() + other.clause.len() + 5);
        new_clause.push_str(&self.clause);
        new_clause.push_str(" AND ");
        new_clause.push_str(&other.clause);
        self.clause = new_clause;
        self.values.extend(other.values);
        self
    }

    /// Combines multiple FilterClause using OR connection.
    pub fn or(mut self, other: FilterClause<T>) -> Self {
        let mut new_clause = String::with_capacity(self.clause.len() + other.clause.len() + 4);
        new_clause.push_str(&self.clause);
        new_clause.push_str(" OR ");
        new_clause.push_str(&other.clause);
        self.clause = new_clause;
        self.values.extend(other.values);
        self
    }
}

/// Simplifies writing, creates a FilterClause for field value comparison query.
pub struct Field<'a, T: Debug + Clone> {
    /// Field name.
    name: &'a str,
    _phantom: PhantomData<T>,
}

impl<'a, T: Debug + Clone> Field<'a, T> {
    /// Creates a new FieldValue instance.
    ///
    /// # Parameters
    /// - `name`: Field name.
    ///
    /// # Returns
    /// - `Field`: Initialized Field instance.
    pub fn get(name: &'a str) -> Self {
        Field { name, _phantom: PhantomData }
    }

    pub fn expr(self, expr: impl Into<String>) -> FilterClause<T> {
        FilterClause::expr(expr)
    }

    /// Creates an equal condition.
    ///
    /// # Parameters
    /// - `value`: Parameter value.
    ///
    /// # Returns
    /// - `FilterClause`: Initialized filter clause builder instance.
    pub fn eq(self, value: impl Into<T>) -> FilterClause<T> 
    {
        FilterClause::new(&self.name, "=", value)
    }

    /// Creates a greater than condition.
    ///
    /// # Parameters
    /// - `value`: Parameter value.
    ///
    /// # Returns
    /// - `FilterClause`: Initialized filter clause builder instance.
    pub fn gt(self, value: impl Into<T>) -> FilterClause<T> {
        FilterClause::new(&self.name, ">", value)
    }

    /// Creates a less than condition.
    ///
    /// # Parameters
    /// - `value`: Parameter value.
    ///
    /// # Returns
    /// - `FilterClause`: Initialized filter clause builder instance.
    pub fn lt(self, value: impl Into<T>) -> FilterClause<T> {
        FilterClause::new(&self.name, "<", value)
    }

    /// Creates a greater than or equal condition.
    ///
    /// # Parameters
    /// - `value`: Parameter value.
    ///
    /// # Returns
    /// - `FilterClause`: Initialized filter clause builder instance.
    pub fn gte(self, value: impl Into<T>) -> FilterClause<T> {
        FilterClause::new(&self.name, ">=", value)
    }

    /// Creates a less than or equal condition.
    ///
    /// # Parameters
    /// - `value`: Parameter value.
    ///
    /// # Returns
    /// - `FilterClause`: Initialized filter clause builder instance.
    pub fn lte(self, value: impl Into<T>) -> FilterClause<T> {
        FilterClause::new(&self.name, "<=", value)
    }

    /// Creates a LIKE condition.
    ///
    /// # Parameters
    /// - `value`: Parameter value.
    ///
    /// # Returns
    /// - `FilterClause`: Initialized filter clause builder instance.
    pub fn like(self, value: impl Into<T>) -> FilterClause<T> {
        FilterClause::new(&self.name, "LIKE", value)
    }

    /// Creates a not equal condition.
    ///
    /// # Parameters
    /// - `value`: Parameter value.
    ///
    /// # Returns
    /// - `FilterClause`: Initialized filter clause builder instance.
    pub fn ne(self, value: impl Into<T>) -> FilterClause<T> {
        FilterClause::new(&self.name, "!=", value)
    }

    /// Creates an IS NULL condition.
    ///
    /// # Returns
    /// - `FilterClause`: Initialized filter clause builder instance.
    pub fn is_null(self) -> FilterClause<T> {
        FilterClause::null_or_not(&self.name, false)
    }

    /// Creates an IS NOT NULL condition.
    ///
    /// # Returns
    /// - `FilterClause`: Initialized filter clause builder instance.
    pub fn is_not_null(self) -> FilterClause<T> {
        FilterClause::null_or_not(&self.name, true)
    }

    /// Creates an IN condition.
    ///
    /// # Parameters
    /// - `values`: Parameter values list.
    ///
    /// # Returns
    /// - `FilterClause`: Initialized filter clause builder instance.
    pub fn r#in<I, U>(self, values: I) -> FilterClause<T>
    where
        I: IntoIterator<Item = U>,
        U: Into<T>,
    {
        FilterClause::in_or_not_in(&self.name, values, false)
    }

    /// Creates a NOT IN condition.
    ///
    /// # Parameters
    /// - `values`: Parameter values list.
    ///
    /// # Returns
    /// - `FilterClause`: Initialized filter clause builder instance.
    pub fn not_in<I, U>(self, values: I) -> FilterClause<T>
    where
        I: IntoIterator<Item = U>,
        U: Into<T>,
    {
        FilterClause::in_or_not_in(&self.name, values, true)
    }

    /// Creates a BETWEEN condition.
    ///
    /// # Parameters
    /// - `value1`: First parameter value.
    /// - `value2`: Second parameter value.
    ///
    /// # Returns
    /// - `FilterClause`: Initialized filter clause builder instance.
    pub fn between(self, value1: impl Into<T>, value2: impl Into<T>) -> FilterClause<T> {
        FilterClause::between(&self.name, value1, value2)
    }
}