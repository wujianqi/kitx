use super::filter::FilterClause;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct Join<'a, T: Debug + Clone> {
    joins: Vec<(String, &'a str, Option<FilterClause<T>>)>, // (join_type, table, condition)
}

impl<'a, T: Debug + Clone> Join<'a, T> {
    /// 创建一个新的 JOIN 实例（私有方法，用于内部逻辑）
    fn new(join_type: &str, table: &'a str) -> Self {
        Join {
            joins: vec![(join_type.to_string(), table, None)],
        }
    }

    /// 添加一个 LEFT JOIN
    pub fn left(mut self, table: &'a str) -> Self {
        if self.joins.is_empty() {
            Self::new("LEFT JOIN", table)
        } else {
            self.joins.push(("LEFT JOIN".to_string(), table, None));
            self
        }
    }

    /// 添加一个 RIGHT JOIN
    pub fn right(mut self, table: &'a str) -> Self {
        if self.joins.is_empty() {
            Self::new("RIGHT JOIN", table)
        } else {
            self.joins.push(("RIGHT JOIN".to_string(), table, None));
            self
        }
    }

    /// 添加一个 INNER JOIN
    pub fn inner(mut self, table: &'a str) -> Self {
        if self.joins.is_empty() {
            Self::new("INNER JOIN", table)
        } else {
            self.joins.push(("INNER JOIN".to_string(), table, None));
            self
        }
    }

    /// 添加一个 FULL OUTER JOIN
    pub fn full_outer(mut self, table: &'a str) -> Self {
        if self.joins.is_empty() {
            Self::new("FULL OUTER JOIN", table)
        } else {
            self.joins.push(("FULL OUTER JOIN".to_string(), table, None));
            self
        }
    }

    /// 为最后一个 JOIN 设置连接条件
    pub fn on(mut self, condition: FilterClause<T>) -> Self {
        if let Some(last_join) = self.joins.last_mut() {
            last_join.2 = Some(condition);
        }
        self
    }

    /// 构建所有 JOIN 子句的 SQL 字符串和参数值
    pub fn build(&self) -> (String, Vec<T>)
    where
        T: Clone,
    {
        // 预先分配足够的容量
        let mut sql = String::with_capacity(self.joins.len() * 128); // 根据实际情况调整容量
        let mut values = Vec::new();

        for (join_type, table, condition) in &self.joins {
            sql.push_str(" ");
            sql.push_str(join_type);
            sql.push_str(" ");
            sql.push_str(table);

            if let Some(condition) = condition {
                let (clause, condition_values) = condition.clone().build();
                sql.push_str(" ON ");
                sql.push_str(&clause);
                values.extend(condition_values);
            }
        }

        (sql, values)
    }
}
