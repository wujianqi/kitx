use std::fmt::Debug;

use crate::common::builder::BuilderTrait;

pub struct SqlBuilder<T: Debug + Clone> {
    sql: String,
    values: Vec<T>
}

impl<T: Debug + Clone> SqlBuilder<T> {
    /// Creates a new Builder instance with the given SQL query and parameter values.
    pub fn raw(sql: impl Into<String>, params: Option<Vec<impl Into<T>>>) -> Self 
    {
        let sql = sql.into();
        let mut values = vec![];
        if let Some(vals) = params {
            values.extend(vals.into_iter().map(|v| v.into()));
        }
        Self {
            sql,
            values
        }
    }

    /// Prepends a new SQL query and parameter value to the existing query.
    pub fn prepend(mut self, sql: impl Into<String>, value: Option<impl Into<T>>) -> Self {
        let sql = sql.into();
        let mut values = vec![];
        if let Some(val) = value {
            values.push(val.into());
        }
        self.sql.insert_str(0, &sql);
        self.values.splice(0..0, values);
        self
    }

    /// Appends a new SQL query and parameter value to the existing query.
    pub fn append(mut self, sql: impl Into<String>, value: Option<impl Into<T>>)-> Self {
        let sql = sql.into();
        let mut values = vec![];
        if let Some(val) = value {
            values.push(val.into());
        }
        self.sql.push_str(&sql);
        self.values.extend(values);
        self
    }

}

impl<T: Debug + Clone> BuilderTrait<T> for SqlBuilder<T> {
    fn build(self) -> (String, Vec<T>) {
        (self.sql, self.values)
    }
}
