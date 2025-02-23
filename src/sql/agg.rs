use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct Agg<'a, T: Debug + Clone> {
    aggregates: Vec<(&'a str, &'a str, Option<&'a str>)>, // (function, column, alias)
    group_by_columns: Vec<&'a str>,
    having_conditions: Vec<(&'a str, T)>, // 存储 HAVING 条件和绑定值
    values: Vec<T>,
}

impl<'a, T: Debug + Clone> Agg<'a, T> {
    /// 创建一个新的 `Agg` 实例
    fn new() -> Self {
        Agg {
            aggregates: Vec::new(),
            group_by_columns: Vec::new(),
            having_conditions: Vec::new(),
            values: Vec::new(),
        }
    }

    /// 添加 COUNT 聚合函数
    pub fn count(column: &'a str, alias: Option<&'a str>) -> Self {
        let mut agg = Self::new();
        agg.aggregates.push(("COUNT", column, alias));
        agg
    }

    /// 添加 SUM 聚合函数
    pub fn sum(column: &'a str, alias: Option<&'a str>) -> Self {
        let mut agg = Self::new();
        agg.aggregates.push(("SUM", column, alias));
        agg
    }

    /// 添加 AVG 聚合函数
    pub fn avg(column: &'a str, alias: Option<&'a str>) -> Self {
        let mut agg = Self::new();
        agg.aggregates.push(("AVG", column, alias));
        agg
    }

    /// 添加 MIN 聚合函数
    pub fn min(column: &'a str, alias: Option<&'a str>) -> Self {
        let mut agg = Self::new();
        agg.aggregates.push(("MIN", column, alias));
        agg
    }

    /// 添加 MAX 聚合函数
    pub fn max(column: &'a str, alias: Option<&'a str>) -> Self {
        let mut agg = Self::new();
        agg.aggregates.push(("MAX", column, alias));
        agg
    }

    /// 添加 GROUP BY 子句
    pub fn group_by(mut self, columns: &[&'a str]) -> Self {
        self.group_by_columns.extend_from_slice(columns);
        self
    }

    /// 添加 HAVING 条件，并绑定值
    pub fn having(mut self, condition: &'a str, value: T) -> Self
    where
        T: Clone,
    {
        self.having_conditions.push((condition, value.clone()));
        self.values.push(value); // 存储原始值
        self
    }

    /// 内部方法：将聚合函数添加到 SQL 语句中
    fn add_aggregates_to_sql(&self, sql: &mut String) {
        for (func, column, alias) in &self.aggregates {
            sql.push_str(", ");
            sql.push_str(func);
            sql.push('(');
            sql.push_str(column);
            sql.push(')');
            if let Some(alias) = alias {
                sql.push_str(" AS ");
                sql.push_str(alias);
            }
        }
    }

    /// 内部方法：将 HAVING 条件添加到 SQL 语句中
    fn add_having_to_sql(&self, sql: &mut String) {
        if !self.having_conditions.is_empty() {
            sql.push_str(" HAVING ");
            for (i, (condition, _)) in self.having_conditions.iter().enumerate() {
                if i > 0 {
                    sql.push_str(" AND ");
                }
                sql.push_str(condition);
                sql.push(' ');
                sql.push('?'); // 使用占位符
            }
        }
    }

    /// 构建最终的 SQL 语句和参数值
    pub fn build(self, base_sql: &str) -> (String, Vec<T>) {
        // 预先分配足够的容量
        let mut sql = String::with_capacity(base_sql.len() + self.aggregates.len() * 64 + self.group_by_columns.len() * 20 + self.having_conditions.len() * 30);

        sql.push_str(base_sql);

        // 添加聚合函数
        self.add_aggregates_to_sql(&mut sql);

        // 添加 GROUP BY 子句
        if !self.group_by_columns.is_empty() {
            sql.push_str(" GROUP BY ");
            sql.push_str(&self.group_by_columns.join(", "));
        }

        // 添加 HAVING 子句
        self.add_having_to_sql(&mut sql);

        (sql, self.values)
    }
}
