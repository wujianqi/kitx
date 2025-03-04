use std::marker::PhantomData;
use std::fmt::Debug;
use super::filter::FilterClause;

/// CASE WHEN clause builder, used to create CASE WHEN conditions.
#[derive(Debug, Clone)]
pub struct WhenClause<'a, T: Debug + Clone> {
    /// Stores multiple CASE WHEN clauses.
    cases: Vec<(String, Vec<T>)>,
    /// Currently building CASE WHEN clause.
    current_case: Option<(String, Vec<T>)>,
    /// Lifetime marker, used to reference external strings.
    _marker: PhantomData<&'a str>,
}

impl<'a, T: Debug + Clone> WhenClause<'a, T> {
    /// Starts a new CASE WHEN clause or initializes a new WhenClause instance.
    ///
    /// If there is already a CASE WHEN clause being built, it is saved to `cases` and a new clause is started.
    /// Otherwise, initializes a new WhenClause instance.
    ///
    /// # Returns
    /// - `WhenClause`: Updated WhenClause instance.
    pub fn case() -> Self {
        WhenClause {
            cases: Vec::new(),
            current_case: Some((String::from("CASE"), Vec::new())),
            _marker: std::marker::PhantomData,
        }
    }

    /// Adds a WHEN clause to the current CASE WHEN clause.
    ///
    /// # Parameters
    /// - `condition`: WHEN condition.
    /// - `result`: Value returned when the condition is true.
    ///
    /// # Returns
    /// - `WhenClause`: Updated WhenClause instance.
    pub fn when(mut self, condition: FilterClause<T>, result: &'a str) -> Self {
        if let Some((ref mut case_when_clause, ref mut values)) = self.current_case {
            let (clause, condition_values) = condition.build();
            case_when_clause.push_str(" WHEN ");
            case_when_clause.push_str(&clause);
            case_when_clause.push_str(" THEN ");
            case_when_clause.push_str(result);
            values.extend(condition_values);
        }
        self
    }

    /// Adds an ELSE clause to the current CASE WHEN clause.
    ///
    /// # Parameters
    /// - `result`: Value returned when all conditions are not met.
    ///
    /// # Returns
    /// - `WhenClause`: Updated WhenClause instance.
    pub fn else_result(mut self, result: &'a str) -> Self {
        if let Some((ref mut case_when_clause, _)) = self.current_case {
            case_when_clause.push_str(" ELSE ");
            case_when_clause.push_str(result);
        }
        self
    }

    /// Builds all CASE WHEN clauses.
    ///
    /// # Returns
    /// - `(String, Vec<T>)`: Concatenated CASE WHEN clause string and parameter values list.
    pub fn build(mut self) -> (String, Vec<T>) {
        if let Some(current_case) = self.current_case.take() {
            self.cases.push(current_case);
        }

        // Pre-allocate sufficient capacity
        let mut sql = String::with_capacity(self.cases.len() * 64); // Adjust capacity as needed
        let mut values = Vec::new();

        for (case_when_clause, condition_values) in self.cases {
            sql.push_str(&case_when_clause);
            sql.push_str(" END, ");
            values.extend(condition_values);
        }

        // Remove the last extra comma and space
        if sql.ends_with(", ") {
            sql.truncate(sql.len() - 2);
        }

        (sql, values)
    }
}