use std::fmt::Debug;

/// SQL 构建器 trait，定义通用的 SQL 构建方法。
pub trait BuilderTrait<T: Debug + Clone> {
    type FilterClause;
    type WhenClause<'a>;
    type Join<'a>;
    type Agg<'a>;
    
    /// 创建一个新的 Builder 实例。
    fn new(sql: String, values: Option<Vec<T>>) -> Self;

    /// 创建一个新的 SELECT 语句。
    fn select(table: impl Into<String>, columns: &[&str]) -> Self;

    /// 创建一个新的 INSERT INTO 语句。
    fn insert_into(table: &str, columns: &[&str], values: Vec<Vec<T>>) -> Self;

    /// 创建一个新的 UPDATE 语句。
    fn update(table: &str, columns: &[&str], values: Vec<T>) -> Self;

    /// 创建一个新的 DELETE 语句。
    fn delete(table: &str) -> Self;

    /// 添加 WHERE 子句。
    fn filter(&mut self, clause: Self::FilterClause) -> &mut Self;

    /// 添加 WHERE …… OR 子句。
    fn or(&mut self, clause: Self::FilterClause) -> &mut Self;

    /// 添加 ORDER BY 子句。
    fn order_by(&mut self, column: &str, asc: bool) -> &mut Self;

    /// 添加 LIMIT 和 OFFSET 子句。
    fn limit_offset(&mut self, limit: u64, offset: Option<u64>) -> &mut Self;

    /// 添加子查询嵌套使用。
    fn add_subquery(&mut self, builder: Self) -> &mut Self;

    /// 添加自定义 SQL。
    fn append(&mut self, sql: &str, bind_values: Option<Vec<T>>) -> &mut Self;

    /// 挂接 CASE WHEN 子句到 SQL 语句中。
    fn case_when<'a> (&mut self, case_when: Self::WhenClause<'a> ) -> &mut Self;

    /// 挂接 JOIN 子句到 SQL 语句中。
    fn join<'a> (&mut self, join: Self::Join<'a> ) -> &mut Self;

    /// 挂接聚合查询到 SQL 语句中。
    fn aggregate<'a> (&mut self, agg: Self::Agg<'a> ) -> &mut Self;

    /// 构建最终的 SQL 字符串和绑定参数，不可变引用输出。
    fn build(self) -> (String, Vec<T>);

    /// 构建最终的 SQL 字符串和绑定参数，可变引用输出。
    fn build_mut(&mut self) -> (String, Vec<T>);
}

/// 用于封装查询条件的结构体
pub struct BuilderCondition<'a, T: Debug + Clone> {
    condition: Option<Box<dyn Fn(&mut T) + Send + 'a>>,
}

impl<'a, T> BuilderCondition<'a, T> 
where
    T: Debug + Clone,
{
    /// 创建一个新的 BuilderCondition
    /// # 参数
    /// - `query_fn`: 查询条件的函数，接受一个Builder 参数，并返回一个 BuilderCondition
    ///
    /// # 返回
    pub fn from<F>(query_fn: F) -> Self 
    where
        F: Fn(&mut T) + Send + 'a,
    {
        BuilderCondition {
            condition: Some(Box::new(query_fn)),
        }
    }

    /// 创建一个没有任何查询条件的 Query
    pub fn empty() -> Self {
        BuilderCondition { condition: None }
    }

    /// 将查询条件应用到 Builder
    pub fn apply(&self, builder: &mut T) {
        if let Some(ref query_fn) = self.condition {
            query_fn(builder);
        }
    }
}
