use std::marker::PhantomData;
use std::fmt::Debug;
use super::filter::FilterClause;

/// CASE WHEN 子句构建器，用于创建 CASE WHEN 条件。
#[derive(Debug, Clone)]
pub struct WhenClause<'a, T: Debug + Clone> {
    /// 存储多个 CASE WHEN 子句。
    cases: Vec<(String, Vec<T>)>,
    /// 当前正在构建的 CASE WHEN 子句。
    current_case: Option<(String, Vec<T>)>,
    /// 生命周期标记，用于引用外部字符串。
    _marker: PhantomData<&'a str>,
}

impl<'a, T: Debug + Clone> WhenClause<'a, T> {
    /// 开始一个新的 CASE WHEN 子句或初始化一个新的 WhenClause 实例。
    ///
    /// 如果当前已经有一个 CASE WHEN 子句正在构建，则将其保存到 `cases` 中，并开始一个新的子句。
    /// 否则，初始化一个新的 WhenClause 实例。
    ///
    /// # 返回
    /// - `WhenClause`: 更新后的 WhenClause 实例。
    pub fn case() -> Self {
        WhenClause {
            cases: Vec::new(),
            current_case: Some((String::from("CASE"), Vec::new())),
            _marker: std::marker::PhantomData,
        }
    }

    /// 添加 WHEN 子句到当前的 CASE WHEN 子句中。
    ///
    /// # 参数
    /// - `condition`: WHEN 条件。
    /// - `result`: 条件为真时返回的值。
    ///
    /// # 返回
    /// - `WhenClause`: 更新后的 WhenClause 实例。
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

    /// 添加 ELSE 子句到当前的 CASE WHEN 子句中。
    ///
    /// # 参数
    /// - `result`: 所有条件都不满足时返回的值。
    ///
    /// # 返回
    /// - `WhenClause`: 更新后的 WhenClause 实例。
    pub fn else_result(mut self, result: &'a str) -> Self {
        if let Some((ref mut case_when_clause, _)) = self.current_case {
            case_when_clause.push_str(" ELSE ");
            case_when_clause.push_str(result);
        }
        self
    }

    /// 构建所有的 CASE WHEN 子句。
    ///
    /// # 返回
    /// - `(String, Vec<T>)`: 拼接后的 CASE WHEN 子句字符串和参数值列表。
    pub fn build(mut self) -> (String, Vec<T>) {
        if let Some(current_case) = self.current_case.take() {
            self.cases.push(current_case);
        }

        // 预先分配足够的容量
        let mut sql = String::with_capacity(self.cases.len() * 64); // 根据实际情况调整容量
        let mut values = Vec::new();

        for (case_when_clause, condition_values) in self.cases {
            sql.push_str(&case_when_clause);
            sql.push_str(" END, ");
            values.extend(condition_values);
        }

        // 移除最后一个多余的逗号和空格
        if sql.ends_with(", ") {
            sql.truncate(sql.len() - 2);
        }

        (sql, values)
    }
}