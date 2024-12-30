use std::marker::PhantomData;

/// SQL 构建器，用于逐步构建最终的 SQL 语句。
#[derive(Debug, Clone)]
pub struct Builder<T> {
    /// SQL 语句字符串。
    sql: String,
    /// WHERE 子句及其对应的参数值列表。
    where_clauses: Vec<(String, Vec<T>)>, // (clause, values)
    /// ORDER BY 子句及其排序方式。
    order_by_clauses: Vec<(String, bool)>, // (column, asc)
    /// LIMIT 子句的限制数量。
    limit_clause: Option<i64>,
    /// OFFSET 子句的偏移量。
    offset_clause: Option<i64>,
    /// 收集所有参数值。
    values: Vec<T>,
}

impl<T> Builder<T> {
    /// 创建一个新的 Builder 实例。
    ///
    /// # 参数
    /// - `sql`: SQL 语句字符串。
    ///
    /// # 返回
    /// - `Builder`: SQL 构建器实例。
    pub fn new(sql: String) -> Self {
        Builder {
            sql,
            where_clauses: Vec::new(),
            order_by_clauses: Vec::new(),
            limit_clause: None,
            offset_clause: None,
            values: Vec::new(),
        }
    }

    /// 创建一个新的 SELECT 语句。
    ///
    /// # 参数
    /// - `table`: 表名。
    /// - `columns`: 要查询的列名列表。如果为空，则查询所有列 (`*`)。
    ///
    /// # 返回
    /// - `Builder`: SQL 构建器实例。
    pub fn select(table: &str, columns: &[&str]) -> Self {
        let cols = if columns.is_empty() { "*" } else { &columns.join(", ") };
        let sql = format!("SELECT {} FROM {}", cols, table);
        Builder::new(sql)
    }

    /// 创建一个新的 INSERT INTO 语句。
    ///
    /// # 参数
    /// - `table`: 表名。
    /// - `columns`: 列名列表。
    /// - `values`: 每行数据的值列表。
    ///
    /// # 返回
    /// - `Builder`: SQL 构建器实例。
    pub fn insert_into(table: &str, columns: &[&str], values: Vec<Vec<T>>) -> Self {
        let mut cols_values = Vec::new();
        let mut sql = format!("INSERT INTO {} ( {} ) VALUES ", table, columns.join(", "));

        for row in values {
            let placeholders = vec!["?"; row.len()].join(", ");
            sql.push_str(&format!("( {} )", placeholders));
            cols_values.extend(row);
        }

        Builder {
            sql,
            where_clauses: Vec::new(),
            values: cols_values,
            order_by_clauses: Vec::new(),
            limit_clause: None,
            offset_clause: None,
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
    /// - `Builder`: SQL 构建器实例。
    pub fn update(table: &str, columns: &[&str], values: Vec<T>) -> Self {
        let set_clause = columns
            .iter()
            .map(|col| format!("{} = ?", col))
            .collect::<Vec<String>>()
            .join(", ");

        let sql = format!("UPDATE {} SET {}", table, set_clause);

        Builder {
            sql,
            where_clauses: Vec::new(),
            values,
            order_by_clauses: Vec::new(),
            limit_clause: None,
            offset_clause: None,
        }
    }

    /// 添加一个 CASE WHEN 子句到 UPDATE 语句中。
    ///
    /// # 参数
    /// - `column`: 要更新的列名。
    /// - `cases`: 条件表达式列表，每个条件表达式包含一个条件和一个值。
    /// - `else_value`: 当所有条件都不满足时的默认值。
    ///
    /// # 返回
    /// - `Builder`: 更新后的 SQL 构建器实例。
    pub fn case_when(mut self, column: &str, cases: Vec<(WhereClause<T>, T)>, else_value: T) -> Self {
        let mut case_clause = format!("{} = CASE ", column);

        for (condition, value) in cases {
            let (condition_sql, condition_values) = condition.build();
            case_clause.push_str(&format!("WHEN {} THEN ? ", condition_sql));
            self.values.extend(condition_values);
            self.values.push(value);
        }

        case_clause.push_str(&format!("ELSE ? END"));
        self.values.push(else_value);

        // 将 CASE WHEN 子句添加到 SQL 语句中
        self.sql.push_str(&format!(", {}", case_clause));

        self
    }

    /// 创建一个新的 DELETE 语句。
    ///
    /// # 参数
    /// - `table`: 表名。
    ///
    /// # 返回
    /// - `Builder`: SQL 构建器实例。
    pub fn delete(table: &str) -> Self {
        Builder::new(format!("DELETE FROM {}", table))
    }

    /// 添加单一的 WHERE 查询条件。
    ///
    /// # 参数
    /// - `clause_builder`: WHERE 子句构建器。
    ///
    /// # 返回
    /// - `Builder`: 更新后的 SQL 构建器实例。
    pub fn filter(mut self, clause_builder: WhereClause<T>) -> Self {
        let (sql, values) = clause_builder.build();
        self.where_clauses.push((sql, values));
        self
    }

    /// 添加多个 WHERE 查询条件，使用 AND 或 OR 连接。
    ///
    /// # 参数
    /// - `clause_builders`: WHERE 子句构建器列表。
    /// - `use_or`: 是否使用 OR 连接，默认为 AND。
    ///
    /// # 返回
    /// - `Builder`: 更新后的 SQL 构建器实例。
    pub fn combine(mut self, clause_builders: Vec<WhereClause<T>>, use_or: bool) -> Self {
        let connector = if use_or { " OR " } else { " AND " };
        let mut sql_parts = Vec::new();
        let mut values = Vec::new();

        for clause_builder in clause_builders {
            let (sql, vals) = clause_builder.build();
            sql_parts.push(sql);
            values.extend(vals);
        }

        // 如果有多个部分，则用括号包裹整个组合条件
        let combined_sql = if sql_parts.len() > 1 {
            format!("({})", sql_parts.join(connector))
        } else {
            sql_parts.join(connector)
        };

        self.where_clauses.push((combined_sql, values));
        self
    }

    /// 添加 LIMIT 子句。
    ///
    /// # 参数
    /// - `value`: 限制的数量。
    ///
    /// # 返回
    /// - `Builder`: 更新后的 SQL 构建器实例。
    pub fn limit(mut self, value: i64) -> Self {
        self.limit_clause = Some(value);
        self
    }

    /// 添加分页查询子句。
    ///
    /// # 参数
    /// - `page`: 当前页码。
    /// - `page_size`: 每页显示的记录数。
    ///
    /// # 返回
    /// - `Builder`: 更新后的 SQL 构建器实例。
    pub fn paginate(mut self, page: i64, page_size: i64) -> Self {
        self.limit_clause = Some(page_size);
        self.offset_clause = Some((page - 1) * page_size);
        self
    }

    /// 添加 ORDER BY 子句。
    ///
    /// # 参数
    /// - `column`: 排序的列名。
    /// - `asc`: 是否按升序排列，默认为升序。
    ///
    /// # 返回
    /// - `Builder`: 更新后的 SQL 构建器实例。
    pub fn order_by(mut self, column: &str, asc: bool) -> Self {
        // 尝试找到已有的相同列的排序规则，并移除它
        self.order_by_clauses
            .retain(|(col, _)| col.as_str() != column);

        // 添加新的或更新的排序规则
        self.order_by_clauses.push((column.to_string(), asc));

        self
    }

    /// 私有的辅助函数，用于添加 AND/OR 条件。
    fn add_logical_operator(mut self, clause_builder: WhereClause<T>, connector: &str) -> Self {
        let (sql, values) = clause_builder.build();
        self.where_clauses.push((format!("{} {}", connector, sql), values));
        self
    }

    /// 添加 AND 条件。
    ///
    /// # 参数
    /// - `clause_builder`: WHERE 子句构建器。
    ///
    /// # 返回
    /// - `Builder`: 更新后的 SQL 构建器实例。
    pub fn and(self, clause_builder: WhereClause<T>) -> Self {
        self.add_logical_operator(clause_builder, "AND")
    }

    /// 添加 OR 条件。
    ///
    /// # 参数
    /// - `clause_builder`: WHERE 子句构建器。
    ///
    /// # 返回
    /// - `Builder`: 更新后的 SQL 构建器实例。
    pub fn or(self, clause_builder: WhereClause<T>) -> Self {
        self.add_logical_operator(clause_builder, "OR")
    }

    /// 添加一个子查询到 WHERE 子句。
    ///
    /// # 参数
    /// - `column`: 列名。
    /// - `operator`: 操作符（如 `=`、`IN` 等）。
    /// - `subquery`: 子查询的 SQL 构建器。
    /// - `use_or`: 是否使用 OR 连接，默认为 AND。
    ///
    /// # 返回
    /// - `Builder`: 更新后的 SQL 构建器实例。
    pub fn subquery(mut self, column: &str, operator: &str, subquery: Builder<T>, use_or: bool) -> Self {
        let (sql, values) = subquery.build();
        let subquery_sql = format!("{} {} ({})", column, operator, sql);
        let connector = if use_or {" OR "} else {" AND "};
        self.where_clauses.push((connector.to_owned() + &subquery_sql, values));
        self
    }

    /// 创建一个新的 COUNT 查询。
    ///
    /// # 参数
    /// - `table`: 表名。
    ///
    /// # 返回
    /// - `Builder`: SQL 构建器实例。
    pub fn count(table: &str) -> Self {
        Builder::new(format!("SELECT COUNT(*) FROM {}", table))
    }
    
    /// 创建一个新的聚合函数查询。
    ///
    /// # 参数
    /// - `table`: 表名。
    /// - `agg_function`: 聚合函数名称。
    /// - `column`: 要进行聚合操作的列名。
    ///
    /// # 返回
    /// - `Builder`: SQL 构建器实例。
    pub fn agg(table: &str, agg_function: AggregateFunction, column: &str) -> Self {
        let agg_str = match agg_function {
            AggregateFunction::Sum => "SUM",
            AggregateFunction::Avg => "AVG",
            AggregateFunction::Min => "MIN",
            AggregateFunction::Max => "MAX",
            AggregateFunction::Count => "COUNT",
        };

        Builder::new(format!("SELECT {}({}) FROM {}", agg_str, column, table))
    }

    /// 添加一个 JOIN 子句。
    ///
    /// # 参数
    /// - `join_type`: JOIN 的类型（如 "INNER JOIN", "LEFT JOIN", "RIGHT JOIN", "FULL OUTER JOIN"）。
    /// - `table`: 要连接的表名。
    /// - `condition`: 连接条件。
    ///
    /// # 返回
    /// - `Builder`: 更新后的 SQL 构建器实例。
    pub fn join(mut self, join_type: JoinType, table: &str, condition: &str) -> Self {
        let join_str = match join_type {
            JoinType::InnerJoin => "INNER JOIN",
            JoinType::LeftJoin => "LEFT JOIN",
            JoinType::RightJoin => "RIGHT JOIN",
            JoinType::FullOuterJoin => "FULL OUTER JOIN",
        };
        self.sql.push_str(&format!(" {} {} ON {}", join_str, table, condition));
        self
    }

    /// 获取最终构建好的 SQL 字符串。
    ///
    /// # 返回
    /// - `(String, Vec<T>)`: 最终的 SQL 字符串和参数值列表。
    pub fn build(self) -> (String, Vec<T>) {
        let mut sql = self.sql;
        let mut all_values = self.values;

        // Add WHERE clauses if any
        if !self.where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            let mut first = true;
            for (clause, values) in self.where_clauses {
                if !first {
                    sql.push_str(" ");
                }
                sql.push_str(&clause);
                all_values.extend(values);
                first = false;
            }
        }

        // Add ORDER BY clause if any
        if !self.order_by_clauses.is_empty() {
            sql.push_str(" ORDER BY ");
            let clauses: Vec<String> = self.order_by_clauses
                .into_iter()
                .map(|(col, asc)| format!("{} {}", col, if asc { "ASC" } else { "DESC" }))
                .collect();
            sql.push_str(&clauses.join(", "));
        }

        // Add LIMIT and OFFSET clauses if any
        if let Some(limit) = self.limit_clause {
            sql.push_str(&format!(" LIMIT {}", limit));
            if let Some(offset) = self.offset_clause {
                sql.push_str(&format!(" OFFSET {}", offset));
            }
        }

        (sql, all_values)
    }
}

// 定义 JoinType 枚举
#[derive(Debug, Clone)]
pub enum JoinType {
    InnerJoin,
    LeftJoin,
    RightJoin,
    FullOuterJoin,
}

// 定义 AggregateFunction 枚举
#[derive(Debug, Clone)]
pub enum AggregateFunction {
    Sum,
    Avg,
    Min,
    Max,
    Count,
}

/// WHERE 子句构建器，用于创建 WHERE 条件（单条件和参数值）。
#[derive(Default, Debug, Clone)]
pub struct WhereClause<T> {
    /// 存储条件字符串。
    clause: String,
    /// 存储参数值。
    values: Vec<T>,
}

impl<T> WhereClause<T> {
    /// 创建带有特定操作符的新 Where 构建器。
    ///
    /// # 参数
    /// - `column`: 列名。
    /// - `op`: 操作符（如 `=`、`>`、`<` 等）。
    /// - `value`: 参数值，需实现 `Into<T>`。
    ///
    /// # 返回
    /// - `WhereClause`: 初始化的 WHERE 子句构建器实例。
    fn with<U>(column: &str, op: &str, value: U) -> Self
    where
        U: Into<T>,
    {
        WhereClause {
            clause: format!("{} {} ?", column, op),
            values: vec![value.into()],
        }
    }

    /// 创建 IS NULL 或 IS NOT NULL 查询条件。
    ///
    /// # 参数
    /// - `column`: 列名。
    /// - `not`: 是否为 `IS NOT NULL`，默认为 `IS NULL`。
    ///
    /// # 返回
    /// - `WhereClause`: 初始化的 WHERE 子句构建器实例。
    fn null_or_not(column: &str, not: bool) -> Self {
        let operator = if not { "IS NOT NULL" } else { "IS NULL" };
        WhereClause {
            clause: format!("{} {}", column, operator),
            values: Vec::new(),
        }
    }

    /// 创建一个 IN 或 NOT IN 查询条件。
    ///
    /// # 参数
    /// - `column`: 列名。
    /// - `values`: 参数值列表，需实现 `Into<T>`。
    /// - `not`: 是否为 `NOT IN`，默认为 `IN`。
    ///
    /// # 返回
    /// - `WhereClause`: 初始化的 WHERE 子句构建器实例。
    fn in_or_not_in<I, U>(column: &str, values: I, not: bool) -> Self
    where
        I: IntoIterator<Item = U>,
        U: Into<T>,
    {
        let converted_values: Vec<T> = values.into_iter().map(|v| v.into()).collect();
        let placeholders = vec!["?"; converted_values.len()].join(", ");
        let operator = if not { "NOT IN" } else { "IN" };

        WhereClause {
            clause: format!("{} {} ({})", column, operator, placeholders),
            values: converted_values,
        }
    }

    /// 创建一个 BETWEEN 查询条件。
    ///
    /// # 参数
    /// - `column`: 列名。
    /// - `value1`: 第一个参数值，需实现 `Into<T>`。
    /// - `value2`: 第二个参数值，需实现 `Into<T>`。
    ///
    /// # 返回
    /// - `WhereClause`: 初始化的 WHERE 子句构建器实例。
    fn between<U, V>(column: &str, value1: U, value2: V) -> Self
    where
        U: Into<T>,
        V: Into<T>,
    {
        WhereClause {
            clause: format!("{} BETWEEN ? AND ?", column),
            values: vec![value1.into(), value2.into()],
        }
    }

    /// 获取 WHERE 子句字符串。
    ///
    /// # 返回
    /// - `(String, Vec<T>)`: WHERE 子句字符串和参数值列表。
    fn build(self) -> (String, Vec<T>) {
        (self.clause, self.values)
    }
}

/// 用于简化拼写，按字段项值创建一个 WhereClause 进行比对查询。
pub struct FieldValue<'a, T> {
    /// 字段名称。
    name: &'a str,
    /// 泛型参数值类型。
    _phantom: PhantomData<T>,
}

impl<'a, T> FieldValue<'a, T> {
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
    /// - `WhereClause`: 初始化的 WHERE 子句构建器实例。
    pub fn eq(self, value: impl Into<T>) -> WhereClause<T> 
    {
        WhereClause::with(&self.name, "=", value)
    }

    /// 创建大于条件。
    ///
    /// # 参数
    /// - `value`: 参数值。
    ///
    /// # 返回
    /// - `WhereClause`: 初始化的 WHERE 子句构建器实例。
    pub fn gt(self, value: impl Into<T>) -> WhereClause<T> {
        WhereClause::with(&self.name, ">", value)
    }

    /// 创建小于条件。
    ///
    /// # 参数
    /// - `value`: 参数值。
    ///
    /// # 返回
    /// - `WhereClause`: 初始化的 WHERE 子句构建器实例。
    pub fn lt(self, value: impl Into<T>) -> WhereClause<T> {
        WhereClause::with(&self.name, "<", value)
    }

    /// 创建大于等于条件。
    ///
    /// # 参数
    /// - `value`: 参数值。
    ///
    /// # 返回
    /// - `WhereClause`: 初始化的 WHERE 子句构建器实例。
    pub fn gte(self, value: impl Into<T>) -> WhereClause<T> {
        WhereClause::with(&self.name, ">=", value)
    }

    /// 创建小于等于条件。
    ///
    /// # 参数
    /// - `value`: 参数值。
    ///
    /// # 返回
    /// - `WhereClause`: 初始化的 WHERE 子句构建器实例。
    pub fn lte(self, value: impl Into<T>) -> WhereClause<T> {
        WhereClause::with(&self.name, "<=", value)
    }

    /// 创建 LIKE 条件。
    ///
    /// # 参数
    /// - `value`: 参数值。
    ///
    /// # 返回
    /// - `WhereClause`: 初始化的 WHERE 子句构建器实例。
    pub fn like(self, value: impl Into<T>) -> WhereClause<T> {
        WhereClause::with(&self.name, "LIKE", value)
    }

    /// 创建不等于条件。
    ///
    /// # 参数
    /// - `value`: 参数值。
    ///
    /// # 返回
    /// - `WhereClause`: 初始化的 WHERE 子句构建器实例。
    pub fn ne(self, value: impl Into<T>) -> WhereClause<T> {
        WhereClause::with(&self.name, "!=", value)
    }

    /// 创建 IS NULL 条件。
    ///
    /// # 返回
    /// - `WhereClause`: 初始化的 WHERE 子句构建器实例。
    pub fn is_null(self) -> WhereClause<T> {
        WhereClause::null_or_not(&self.name, false)
    }

    /// 创建 IS NOT NULL 条件。
    ///
    /// # 返回
    /// - `WhereClause`: 初始化的 WHERE 子句构建器实例。
    pub fn is_not_null(self) -> WhereClause<T> {
        WhereClause::null_or_not(&self.name, true)
    }

    /// 创建 IN 条件。
    ///
    /// # 参数
    /// - `values`: 参数值列表。
    ///
    /// # 返回
    /// - `WhereClause`: 初始化的 WHERE 子句构建器实例。
    pub fn in_list<I, U>(self, values: I) -> WhereClause<T>
    where
        I: IntoIterator<Item = U>,
        U: Into<T>,
    {
        WhereClause::in_or_not_in(&self.name, values, false)
    }

    /// 创建 NOT IN 条件。
    ///
    /// # 参数
    /// - `values`: 参数值列表。
    ///
    /// # 返回
    /// - `WhereClause`: 初始化的 WHERE 子句构建器实例。
    pub fn not_in_list<I, U>(self, values: I) -> WhereClause<T>
    where
        I: IntoIterator<Item = U>,
        U: Into<T>,
    {
        WhereClause::in_or_not_in(&self.name, values, true)
    }

    /// 创建 BETWEEN 条件。
    ///
    /// # 参数
    /// - `value1`: 第一个参数值。
    /// - `value2`: 第二个参数值。
    ///
    /// # 返回
    /// - `WhereClause`: 初始化的 WHERE 子句构建器实例。
    pub fn between(self, value1: impl Into<T>, value2: impl Into<T>) -> WhereClause<T> {
        WhereClause::between(&self.name, value1, value2)
    }
}
