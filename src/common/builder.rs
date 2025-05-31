/* use super::types::OrderBy; */

/// SQL builder trait, defining common SQL building methods.
pub trait BuilderTrait<T> {
    /// Build the final SQL query and parameters.
    fn build(self) -> (String, Vec<T>);
}

/// Filter clause trait, extending BuilderTrait with common where clause methods.
pub trait FilterTrait<T>: BuilderTrait<T> {
    type Expr;

    /// Add an AND condition to the WHERE clause.
    fn and_where_mut<F>(&mut self, filter: F) -> &mut Self
    where
        F: Into<Self::Expr>;

    /// Add an OR condition to the WHERE clause.
    fn or_where_mut<F>(&mut self, filter: F) -> &mut Self
    where
        F: Into<Self::Expr>;
}

/* /// Select clause trait, extending FilterTrait with select-specific methods.
pub trait QueryTrait<T>: FilterTrait<T> {
    /// Add an ORDER BY clause.
    fn order_by_mut(&mut self, column: &str, ordering: OrderBy) -> &mut Self;

    /// Add LIMIT and OFFSET clauses.
    fn limit_offset_mut(
        &mut self,
        limit: impl Into<T>,
        offset: Option<impl Into<T>>,
    ) -> &mut Self;
}
 */