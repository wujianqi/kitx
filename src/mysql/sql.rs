use crate::common::sql::{Builder, FieldValue};
pub use crate::common::sql::{WhereClause, AggregateFunction, JoinType};
use super::kind::DataKind;

/// MySQL 专用的 SQL 构建器。
pub struct SQLBuilder<'a> {
    inner: Builder<DataKind<'a>>,
}

impl<'a> SQLBuilder<'a> {
    /// 创建一个新的 SELECT 语句。
    ///
    /// # 参数
    /// - `table`: 表名。
    /// - `columns`: 要查询的列名列表。如果为空，则查询所有列 (`*`)。
    ///
    /// # 返回
    /// - `SQLBuilder`: MySQL 专用的 SQL 构建器实例。
    pub fn select(table: &str, columns: &[&str]) -> Self {
        SQLBuilder {
            inner: Builder::select(table, columns),
        }
    }

    /// 创建一个新的 INSERT INTO 语句。
    ///
    /// # 参数
    /// - `table`: 表名。
    /// - `columns`: 列名列表。
    /// - `values`: 每行数据的值列表。
    ///
    /// # 返回
    /// - `SQLBuilder`: MySQL 专用的 SQL 构建器实例。
    pub fn insert_into(table: &str, columns: &[&str], values: Vec<Vec<DataKind<'a>>>) -> Self {
        SQLBuilder {
            inner: Builder::insert_into(table, columns, values),
        }
    }

    /// 创建一个新的 UPDATE 语句。
    ///
    /// # 参数
    /// - `table`: 表名。
    /// - `columns`: 列名列表。
    /// - `values`: 对应列的更新值。
    ///
    /// # 返回
    /// - `SQLBuilder`: MySQL 专用的 SQL 构建器实例。
    pub fn update(table: &str, columns: &[&str], values: Vec<DataKind<'a>>) -> Self {
        SQLBuilder {
            inner: Builder::update(table, columns, values),
        }
    }

    /// - `SQLBuilder`: 更新后的 SQL 构建器实例。
    pub fn case_when(
        mut self,
        column: &str,
        cases: Vec<(WhereClause<DataKind<'a>>, DataKind<'a>)>,
        else_value: DataKind<'a>,
    ) -> Self {
        // 调用 inner 的 case_when 方法
        self.inner = self.inner.case_when(column, cases, else_value);
        self
    }

    /// 创建一个新的 DELETE 语句。
    ///
    /// # 参数
    /// - `table`: 表名。
    ///
    /// # 返回
    /// - `SQLBuilder`: MySQL 专用的 SQL 构建器实例。
    pub fn delete(table: &str) -> Self {
        SQLBuilder {
            inner: Builder::delete(table),
        }
    }

    /// 添加单一的 WHERE 查询条件。
    ///
    /// # 参数
    /// - `clause_builder`: WHERE 子句构建器。
    ///
    /// # 返回
    /// - `SQLBuilder`: 更新后的 MySQL 专用的 SQL 构建器实例。
    pub fn filter(mut self, clause_builder: WhereClause<DataKind<'a>>) -> Self {
        self.inner = self.inner.filter(clause_builder);
        self
    }

    /// 添加多个 WHERE 查询条件，使用 AND 或 OR 连接。
    ///
    /// # 参数
    /// - `clause_builders`: WHERE 子句构建器列表。
    /// - `use_or`: 是否使用 OR 连接，默认为 AND。
    ///
    /// # 返回
    /// - `SQLBuilder`: 更新后的 MySQL 专用的 SQL 构建器实例。
    pub fn combine(mut self, clause_builders: Vec<WhereClause<DataKind<'a>>>, use_or: bool) -> Self {
        self.inner = self.inner.combine(clause_builders, use_or);
        self
    }

    /// 添加 LIMIT 子句。
    ///
    /// # 参数
    /// - `value`: 限制的数量。
    ///
    /// # 返回
    /// - `SQLBuilder`: 更新后的 MySQL 专用的 SQL 构建器实例。
    pub fn limit(mut self, value: i64) -> Self {
        self.inner = self.inner.limit(value);
        self
    }

    /// 添加分页查询子句。
    ///
    /// # 参数
    /// - `page`: 当前页码。
    /// - `page_size`: 每页显示的记录数。
    ///
    /// # 返回
    /// - `SQLBuilder`: 更新后的 MySQL 专用的 SQL 构建器实例。
    pub fn paginate(mut self, page: i64, page_size: i64) -> Self {
        self.inner = self.inner.paginate(page, page_size);
        self
    }

    /// 添加 ORDER BY 子句。
    ///
    /// # 参数
    /// - `column`: 排序的列名。
    /// - `asc`: 是否按升序排列，默认为升序。
    ///
    /// # 返回
    /// - `SQLBuilder`: 更新后的 MySQL 专用的 SQL 构建器实例。
    pub fn order_by(mut self, column: &str, asc: bool) -> Self {
        self.inner = self.inner.order_by(column, asc);
        self
    }

    /// 添加 AND 条件。
    ///
    /// # 参数
    /// - `clause_builder`: WHERE 子句构建器。
    ///
    /// # 返回
    /// - `SQLBuilder`: 更新后的 MySQL 专用的 SQL 构建器实例。
    pub fn and(self, clause_builder: WhereClause<DataKind<'a>>) -> Self {
        SQLBuilder {
            inner: self.inner.and(clause_builder),
        }
    }

    /// 添加 OR 条件。
    ///
    /// # 参数
    /// - `clause_builder`: WHERE 子句构建器。
    ///
    /// # 返回
    /// - `SQLBuilder`: 更新后的 MySQL 专用的 SQL 构建器实例。
    pub fn or(self, clause_builder: WhereClause<DataKind<'a>>) -> Self {
        SQLBuilder {
            inner: self.inner.or(clause_builder),
        }
    }

    /// 添加一个子查询到 WHERE 子句。
    ///
    /// # 参数
    /// - `column`: 列名。
    /// - `operator`: 操作符（如 `=`、`IN` 等）。
    /// - `subquery`: 子查询的 SQLBuilder 构建器。
    /// - `use_or`: 是否使用 OR 连接，默认为 AND。
    ///
    /// # 返回
    /// - `SQLBuilder`: 更新后的 MySQL 专用的 SQL 构建器实例。
    pub fn subquery(mut self, column: &str, operator: &str, subquery: SQLBuilder<'a>, use_or: bool) -> Self {
        self.inner = self.inner.subquery(column, operator, subquery.inner, use_or);
        self
    }

    /// 创建一个新的 COUNT 查询。
    ///
    /// # 参数
    /// - `table`: 表名。
    ///
    /// # 返回
    /// - `SQLBuilder`: MySQL 专用的 SQL 构建器实例。
    pub fn count(table: &str) -> Self {
        SQLBuilder {
            inner: Builder::count(table),
        }
    }

    /// 创建一个新的聚合函数查询。
    ///
    /// # 参数
    /// - `table`: 表名。
    /// - `agg_function`: 聚合函数名称。
    /// - `column`: 要进行聚合操作的列名。
    ///
    /// # 返回
    /// - `SQLBuilder`: MySQL 专用的 SQL 构建器实例。
    pub fn agg(table: &str, agg_function: AggregateFunction, column: &str) -> Self {
        SQLBuilder {
            inner: Builder::agg(table, agg_function, column),
        }
    }

    /// 添加一个 JOIN 子句。
    ///
    /// # 参数
    /// - `join_type`: JOIN 的类型（如 "INNER JOIN", "LEFT JOIN", "RIGHT JOIN", "FULL OUTER JOIN"）。
    /// - `table`: 要连接的表名。
    /// - `condition`: 连接条件。
    ///
    /// # 返回
    /// - `SQLBuilder`: 更新后的 MySQL 专用的 SQL 构建器实例。
    pub fn join(mut self, join_type: JoinType, table: &str, condition: &str) -> Self {
        self.inner = self.inner.join(join_type, table, condition);
        self
    }

    /// 获取最终构建好的 SQL 字符串。
    ///
    /// # 返回
    /// - `(String, Vec<DataKind<'a>>)`: 最终的 SQL 字符串和参数值列表。
    pub fn build(self) -> (String, Vec<DataKind<'a>>) {
        self.inner.build()
    }
}

/// 创建一个用于获取字段值的对象。
///
/// # 参数
/// - `name`: 字段名。
///
/// # 返回
/// - `FieldValue`: 用于获取字段值的对象。
pub fn field<'a>(name: &'a str) -> FieldValue<'a, DataKind<'a>> {
    FieldValue::get(name)
}