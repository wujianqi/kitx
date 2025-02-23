use std::{fmt::Debug, marker::PhantomData};

/// 过滤查询子句构建器，用于创建查询条件。
#[derive(Default, Debug, Clone)]
pub struct FilterClause<T: Debug + Clone> {
    /// 存储条件字符串。
    clause: String,
    /// 存储参数值。
    values: Vec<T>,
}

impl<T: Debug + Clone> FilterClause<T> {
    /// 创建带有特定操作符的新 Filter 构建器。
    pub fn new<U>(column: &str, op: &str, value: U) -> Self
    where
        U: Into<T>,
    {
        let mut clause = String::with_capacity(column.len() + op.len() + 3); // 预估长度
        clause.push_str(column);
        clause.push_str(" ");
        clause.push_str(op);
        clause.push_str(" ?");
        FilterClause {
            clause,
            values: vec![value.into()],
        }
    }

    /// 创建 IS NULL 或 IS NOT NULL 查询条件。
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

    /// 创建一个 IN 或 NOT IN 查询条件。
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

    /// 创建一个 BETWEEN 查询条件。
    fn between<U, V>(column: &str, value1: U, value2: V) -> Self
    where
        U: Into<T>,
        V: Into<T>,
    {
        let mut clause = String::with_capacity(column.len() + 13); // "BETWEEN ? AND ?" 的长度为 13
        clause.push_str(column);
        clause.push_str(" BETWEEN ? AND ?");
        FilterClause {
            clause,
            values: vec![value1.into(), value2.into()],
        }
    }

    /// 获取 Filter 子句字符串。
    ///
    /// # 返回
    /// - `(String, Vec<T>)`: Filter 子句字符串和参数值列表。
    pub fn build(self) -> (String, Vec<T>) {
        (self.clause, self.values)
    }

    /// 组合多个 FilterClause 使用 AND 连接。
    pub fn and(mut self, other: FilterClause<T>) -> Self {
        let mut new_clause = String::with_capacity(self.clause.len() + other.clause.len() + 5);
        new_clause.push_str(&self.clause);
        new_clause.push_str(" AND ");
        new_clause.push_str(&other.clause);
        self.clause = new_clause;
        self.values.extend(other.values);
        self
    }

    /// 组合多个 FilterClause 使用 OR 连接。
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


/// 用于简化拼写，按字段项值创建一个 FilterClause 进行比对查询。
pub struct FieldValue<'a, T: Debug + Clone> {
    /// 字段名称。
    name: &'a str,
    _phantom: PhantomData<T>,
}

impl<'a, T: Debug + Clone> FieldValue<'a, T> {
    /// 创建一个新的 FieldValue 实例。
    ///
    /// # 参数
    /// - `name`: 字段名称。
    ///
    /// # 返回
    /// - `FieldValue`: 初始化的 FieldValue 实例。
    fn new(name: &'a str) -> Self {
        FieldValue { 
            name,
            _phantom: PhantomData
         }
    }

    /// 公共构造函数。
    pub fn get(name: &'a str) -> Self {
        Self::new(name)
    }

    /// 创建等于条件。
    ///
    /// # 参数
    /// - `value`: 参数值。
    ///
    /// # 返回
    /// - `FilterClause`: 初始化的 Filter 子句构建器实例。
    pub fn eq(self, value: impl Into<T>) -> FilterClause<T> 
    {
        FilterClause::new(&self.name, "=", value)
    }

    /// 创建大于条件。
    ///
    /// # 参数
    /// - `value`: 参数值。
    ///
    /// # 返回
    /// - `FilterClause`: 初始化的 Filter 子句构建器实例。
    pub fn gt(self, value: impl Into<T>) -> FilterClause<T> {
        FilterClause::new(&self.name, ">", value)
    }

    /// 创建小于条件。
    ///
    /// # 参数
    /// - `value`: 参数值。
    ///
    /// # 返回
    /// - `FilterClause`: 初始化的 Filter 子句构建器实例。
    pub fn lt(self, value: impl Into<T>) -> FilterClause<T> {
        FilterClause::new(&self.name, "<", value)
    }

    /// 创建大于等于条件。
    ///
    /// # 参数
    /// - `value`: 参数值。
    ///
    /// # 返回
    /// - `FilterClause`: 初始化的 Filter 子句构建器实例。
    pub fn gte(self, value: impl Into<T>) -> FilterClause<T> {
        FilterClause::new(&self.name, ">=", value)
    }

    /// 创建小于等于条件。
    ///
    /// # 参数
    /// - `value`: 参数值。
    ///
    /// # 返回
    /// - `FilterClause`: 初始化的 Filter 子句构建器实例。
    pub fn lte(self, value: impl Into<T>) -> FilterClause<T> {
        FilterClause::new(&self.name, "<=", value)
    }

    /// 创建 LIKE 条件。
    ///
    /// # 参数
    /// - `value`: 参数值。
    ///
    /// # 返回
    /// - `FilterClause`: 初始化的 Filter 子句构建器实例。
    pub fn like(self, value: impl Into<T>) -> FilterClause<T> {
        FilterClause::new(&self.name, "LIKE", value)
    }

    /// 创建不等于条件。
    ///
    /// # 参数
    /// - `value`: 参数值。
    ///
    /// # 返回
    /// - `FilterClause`: 初始化的 Filter 子句构建器实例。
    pub fn ne(self, value: impl Into<T>) -> FilterClause<T> {
        FilterClause::new(&self.name, "!=", value)
    }

    /// 创建 IS NULL 条件。
    ///
    /// # 返回
    /// - `FilterClause`: 初始化的 Filter 子句构建器实例。
    pub fn is_null(self) -> FilterClause<T> {
        FilterClause::null_or_not(&self.name, false)
    }

    /// 创建 IS NOT NULL 条件。
    ///
    /// # 返回
    /// - `FilterClause`: 初始化的 Filter 子句构建器实例。
    pub fn is_not_null(self) -> FilterClause<T> {
        FilterClause::null_or_not(&self.name, true)
    }

    /// 创建 IN 条件。
    ///
    /// # 参数
    /// - `values`: 参数值列表。
    ///
    /// # 返回
    /// - `FilterClause`: 初始化的 Filter 子句构建器实例。
    pub fn in_list<I, U>(self, values: I) -> FilterClause<T>
    where
        I: IntoIterator<Item = U>,
        U: Into<T>,
    {
        FilterClause::in_or_not_in(&self.name, values, false)
    }

    /// 创建 NOT IN 条件。
    ///
    /// # 参数
    /// - `values`: 参数值列表。
    ///
    /// # 返回
    /// - `FilterClause`: 初始化的 Filter 子句构建器实例。
    pub fn not_in_list<I, U>(self, values: I) -> FilterClause<T>
    where
        I: IntoIterator<Item = U>,
        U: Into<T>,
    {
        FilterClause::in_or_not_in(&self.name, values, true)
    }

    /// 创建 BETWEEN 条件。
    ///
    /// # 参数
    /// - `value1`: 第一个参数值。
    /// - `value2`: 第二个参数值。
    ///
    /// # 返回
    /// - `FilterClause`: 初始化的 Filter 子句构建器实例。
    pub fn between(self, value1: impl Into<T>, value2: impl Into<T>) -> FilterClause<T> {
        FilterClause::between(&self.name, value1, value2)
    }
}
