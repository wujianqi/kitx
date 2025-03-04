use std::fmt::Debug;

/// SQL builder trait, defining common SQL building methods.
pub trait BuilderTrait<T: Debug + Clone> {
    type FilterClause;
    type WhenClause<'a>;
    type Join<'a>;
    type Agg<'a>;
    
    /// Creates a new Builder instance.
    /// 
    /// # Parameters
    /// - `sql`: The initial SQL string.
    /// - `values`: An optional vector of bound parameters.
    /// 
    /// # Returns
    /// A new Builder instance.
    fn new(sql: String, values: Option<Vec<T>>) -> Self;

    /// Creates a new SELECT statement.
    /// 
    /// # Parameters
    /// - `table`: The name of the table to select from.
    /// - `columns`: A slice of column names to select.
    /// 
    /// # Returns
    /// A new Builder instance.
    fn select(table: impl Into<String>, columns: &[&str]) -> Self;

    /// Creates a new INSERT INTO statement.
    /// 
    /// # Parameters
    /// - `table`: The name of the table to insert into.
    /// - `columns`: A slice of column names to insert.
    /// - `values`: A vector of vectors of values to insert.
    /// 
    /// # Returns
    /// A new Builder instance.
    fn insert_into(table: &str, columns: &[&str], values: Vec<Vec<T>>) -> Self;

    /// Creates a new UPDATE statement.
    /// 
    /// # Parameters
    /// - `table`: The name of the table to update.
    /// - `columns`: A slice of column names to update.
    /// - `values`: A vector of values to update.
    /// 
    /// # Returns
    /// A new Builder instance.
    fn update(table: &str, columns: &[&str], values: Vec<T>) -> Self;

    /// Creates a new DELETE statement.
    fn delete(table: &str) -> Self;

    /// Adds a WHERE clause.
    /// 
    /// # Parameters
    /// - `clause`: The WHERE clause to add.
    /// 
    /// # Returns
    /// A reference to the Builder instance.
    fn filter(&mut self, clause: Self::FilterClause) -> &mut Self;

    /// Adds a WHERE ... OR clause.
    /// 
    /// # Parameters
    /// - `clause`: The OR clause to add.
    /// 
    /// # Returns
    /// A reference to the Builder instance.
    fn or(&mut self, clause: Self::FilterClause) -> &mut Self;

    /// Adds an ORDER BY clause.
    /// 
    /// # Parameters
    /// - `column`: The column to order by.
    /// - `asc`: Whether to order by ascending or descending.
    /// 
    /// # Returns
    /// A reference to the Builder instance.
    fn order_by(&mut self, column: &str, asc: bool) -> &mut Self;

    /// Adds LIMIT and OFFSET clauses.
    /// 
    /// # Parameters
    /// - `limit`: The number of rows to limit the result to.
    /// - `offset`: The number of rows to skip.
    /// 
    /// # Returns
    /// A reference to the Builder instance.
    fn limit_offset(&mut self, limit: u64, offset: Option<u64>) -> &mut Self;

    /// Adds a subquery.
    /// 
    /// # Parameters
    /// - `builder`: The Builder instance to add as a subquery.
    /// 
    /// # Returns
    /// A reference to the Builder instance.
    fn add_subquery(&mut self, builder: Self) -> &mut Self;

    /// Appends custom SQL.
    /// 
    /// # Parameters
    /// - `sql`: The SQL string to append.
    /// - `bind_values`: An optional vector of bound parameters to append.
    /// 
    /// # Returns
    /// A reference to the Builder instance.
    fn append(&mut self, sql: &str, bind_values: Option<Vec<T>>) -> &mut Self;

    /// Adds a CASE WHEN clause to the SQL statement.
    /// 
    /// # Parameters
    /// - `case_when`: The CASE WHEN clause to add.
    /// 
    /// # Returns
    /// A reference to the Builder instance.
    fn case_when<'a>(&mut self, case_when: Self::WhenClause<'a>) -> &mut Self;

    /// Adds a JOIN clause to the SQL statement.
    /// 
    /// # Parameters
    /// - `join`: The JOIN clause to add.
    /// 
    /// # Returns
    /// A reference to the Builder instance.
    fn join<'a>(&mut self, join: Self::Join<'a>) -> &mut Self;

    /// Adds an aggregate query to the SQL statement.
    /// 
    /// # Parameters
    /// - `agg`: The aggregate query to add.
    /// 
    /// # Returns
    /// A reference to the Builder instance.
    fn aggregate<'a>(&mut self, agg: Self::Agg<'a>) -> &mut Self;

    /// Builds the final SQL string and bound parameters, immutable reference output.
    /// 
    /// # Returns
    /// A tuple containing the final SQL string and bound parameters.
    fn build(self) -> (String, Vec<T>);

    /// Builds the final SQL string and bound parameters, mutable reference output.
    /// 
    /// # Returns
    /// A tuple containing the final SQL string and bound parameters.
    fn build_mut(&mut self) -> (String, Vec<T>);
}

/// Struct used to encapsulate query conditions.
pub struct BuilderCondition<'a, T: Debug + Clone> {
    condition: Option<Box<dyn Fn(&mut T) + Send + 'a>>,
}

impl<'a, T> BuilderCondition<'a, T> 
where
    T: Debug + Clone,
{
    /// Creates a new BuilderCondition.
    /// 
    /// # Parameters
    /// - `query_fn`: A function representing the query condition, accepts a Builder parameter and returns a BuilderCondition.
    /// 
    /// # Returns
    /// A new BuilderCondition instance.
    pub fn from<F>(query_fn: F) -> Self 
    where
        F: Fn(&mut T) + Send + 'a,
    {
        BuilderCondition {
            condition: Some(Box::new(query_fn)),
        }
    }

    /// Creates an empty BuilderCondition with no query conditions.
    pub fn empty() -> Self {
        BuilderCondition { condition: None }
    }

    /// Applies the query condition to the Builder.
    /// 
    /// # Parameters
    /// - `builder`: The Builder instance to apply the query condition to.
    pub fn apply(&self, builder: &mut T) {
        if let Some(ref query_fn) = self.condition {
            query_fn(builder);
        }
    }
}