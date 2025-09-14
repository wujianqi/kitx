use std::marker::PhantomData;

use crate::common::{error::QueryError, filter::push_primary_key_bind, helper::get_table_name, types::{PrimaryKey, SortOrder}};
use field_access::FieldAccess;
use sqlx::{Database, Encode, Error, QueryBuilder, Type};

/// Select query builder
/// 
/// This struct provides functionality to build SELECT SQL queries.
/// 
/// # Type Parameters
/// * `ET` - Entity type that implements FieldAccess and Default traits
/// * `DB` - Database type that implements sqlx::Database trait
/// * `VAL` - Value type that implements Encode and Type traits
/// 
/// 查询构建器
/// 
/// 该结构体提供了构建 SELECT SQL 查询的功能。
/// 
/// # 类型参数
/// * `ET` - 实现 FieldAccess 和 Default traits 的实体类型
/// * `DB` - 实现 sqlx::Database trait 的数据库类型
/// * `VAL` - 实现 Encode 和 Type traits 的值类型
pub struct Select<'a, ET, DB, VAL>
where
    DB: Database,
{
    query_builder: QueryBuilder<'a, DB>,
    has_from: bool,
    has_filter: bool,
    has_order: bool,
    _phantom: std::marker::PhantomData<(ET, VAL)>,
}

impl<'a, ET, DB, VAL> Select<'a, ET, DB, VAL>
where
    ET: FieldAccess + Default,
    DB: Database,
    VAL: Encode<'a, DB> + Type<DB>,
{

    /// Create a new Select instance
    /// 
    /// # Arguments
    /// * `query_builder` - The QueryBuilder instance to wrap
    /// 
    /// # Returns
    /// A new Select instance
    /// 
    /// 创建新的 Select 实例
    /// 
    /// # 参数
    /// * `query_builder` - 要包装的 QueryBuilder 实例
    /// 
    /// # 返回值
    /// 新的 Select 实例
    fn new(query_builder: QueryBuilder<'a, DB>) -> Self {
        Self { 
            query_builder,
            has_from: false,  
            has_filter: false, 
            has_order: false,
            _phantom: PhantomData
        }
    }

    /// Create a Select instance with default fields
    /// 
    /// # Returns
    /// A new Select instance with default field selection
    /// 
    /// 创建带有默认字段的 Select 实例
    /// 
    /// # 返回值
    /// 带有默认字段选择的新 Select 实例
    pub fn select_default() -> Self {
        let columns = ET::default().field_names().join(", ");
        let mut query_builder = QueryBuilder::new("SELECT ");
        query_builder.push(&columns);
        
        Self::new(query_builder)
    }
    
    /// Create a Select instance with custom fields
    /// 
    /// # Arguments
    /// * `column_build_fn` - Function to build the column selection part of the query
    /// 
    /// # Returns
    /// A new Select instance with custom field selection
    /// 
    /// 创建带有自定义字段的 Select 实例
    /// 
    /// # 参数
    /// * `column_build_fn` - 构建查询中列选择部分的函数
    /// 
    /// # 返回值
    /// 带有自定义字段选择的新 Select 实例
    pub fn select(
        column_build_fn: impl FnOnce(&mut QueryBuilder<'_, DB>),
    ) -> Self {
        let mut query_builder = QueryBuilder::new("SELECT ");
        column_build_fn(&mut query_builder);
   
        Self::new(query_builder)
    }

    /// Add table to the query
    /// 
    /// # Arguments
    /// * `table_name` - Name of the table to query from
    /// 
    /// # Returns
    /// The Select instance with the table added
    /// 
    /// 向查询中添加表
    /// 
    /// # 参数
    /// * `table_name` - 要查询的表名
    /// 
    /// # 返回值
    /// 添加了表的 Select 实例
    fn from_table(mut self, table_name: impl Into<String>) -> Self {
        if self.has_from {
            return self;
        }

        self.query_builder
            .push(" FROM ")
            .push(table_name.into());
        self.has_from = true;
        self
    }

    /// Add default table to the query
    /// 
    /// # Returns
    /// The Select instance with the default table added
    /// 
    /// 向查询中添加默认表
    /// 
    /// # 返回值
    /// 添加了默认表的 Select 实例
    pub fn from_default(self) -> Self {        
        self.from_table(get_table_name::<ET>())
    }

    /// Add a custom table to the query, can include alias, between FROM and WHERE
    /// 
    /// # Arguments
    /// * `table_name` - Name of the table to query from, can include alias
    /// 
    /// # Returns
    /// The Select instance with the table added
    /// 
    /// 向查询中添加自定义表，可以包含别名，介于 FROM 和 WHERE 之间
    /// 
    /// # 参数
    /// * `table_name` - 要查询的表名，可以包含别名
    /// 
    /// # 返回值
    /// 添加了表的 Select 实例
    pub fn from(self, table_name: impl Into<String>) -> Self {
        self.from_table(table_name)
    }

    /// Query by primary key
    /// 
    /// # Arguments
    /// * `primary_key` - Primary key definition
    /// * `primary_value` - Primary key values to match
    /// 
    /// # Returns
    /// A QueryBuilder with the SELECT query or an Error
    /// 
    /// 通过主键查询
    /// 
    /// # 参数
    /// * `primary_key` - 主键定义
    /// * `primary_value` - 要匹配的主键值
    /// 
    /// # 返回值
    /// 包含 SELECT 查询的 QueryBuilder 或错误
    // 修改后的 by_primary_key 方法
    pub fn by_primary_key(
        primary_key: &PrimaryKey<'a>,
        primary_value: &'a Vec<VAL>,
    ) -> Result<QueryBuilder<'a, DB>, Error> {

        let select = Self::select_default().from_default();
        let mut query_builder = select.query_builder;
        
        query_builder.push(" WHERE ");
        push_primary_key_bind::<ET, DB, VAL>(&mut query_builder, &primary_key, primary_value);
            
        Ok(query_builder)
    }
    
    /// Query with custom filter conditions, can include ORDER BY, between WHERE and LIMIT
    /// 
    /// # Arguments
    /// * `filter_build_fn` - Function to build the filter conditions
    /// 
    /// # Returns
    /// The Select instance with the filter conditions added
    /// 
    /// 使用自定义过滤条件查询，介于 WHERE 和 ORDER BY 之间
    /// 
    /// # 参数
    /// * `filter_build_fn` - 构建过滤条件的函数
    /// 
    /// # 返回值
    /// 添加了过滤条件的 Select 实例
    pub fn where_(
        mut self,
        filter_build_fn: impl FnOnce(&mut QueryBuilder<'_, DB>),
    ) -> Self
    {
        let mut query_builder = self.query_builder.push(" WHERE ");
        filter_build_fn(&mut query_builder);
        self.has_filter = true;
        
        self
    }

    /// Add custom sorting to the query using raw SQL text
    /// 
    /// # Arguments
    /// * `order_clause` - Raw SQL order clause (e.g. "created_at DESC, name ASC")
    /// 
    /// # Returns
    /// The Select instance with sorting added
    /// 
    /// 使用原始SQL文本添加自定义排序
    /// 
    /// # 参数
    /// * `order_clause` - 原始SQL排序子句（如"created_at DESC, name ASC"）
    /// 
    /// # 返回值
    /// 添加了排序的Select实例
    pub fn order_by(mut self, order_clause: impl Into<String>) -> Self {
        if self.has_order {
            return self; // 防止重复添加
        }

        self.query_builder
            .push(" ORDER BY ")
            .push(order_clause.into());
        
        self.has_order = true;
        
        self
    }

    /// Add pagination to the query
    /// 
    /// # Arguments
    /// * `page_number` - Page number (starting from 1)
    /// * `page_size` - Number of records per page
    /// 
    /// # Type Parameters
    /// * `VAL` - Must also implement `From<i64>` trait
    /// 
    /// # Returns
    /// A QueryBuilder with the SELECT query or an Error
    /// 
    /// 向查询中添加分页
    /// 
    /// # 参数
    /// * `page_number` - 页码（从1开始）
    /// * `page_size` - 每页记录数
    /// 
    /// # 类型参数
    /// * `VAL` - 还必须实现 `From<i64>` trait
    /// 
    /// # 返回值
    /// 包含 SELECT 查询的 QueryBuilder 或错误
    pub fn paginate(
        mut self,
        page_number: u64,
        page_size: u64,
    ) -> Result<QueryBuilder<'a, DB>, Error>
    where
        VAL: Encode<'a, DB> + Type<DB> + From<i64> + 'a,
    {
        if page_size == 0 || page_number < 1 {
            return Err(QueryError::PageNumberInvalid.into());
        }

        let offset = ((page_number - 1) * page_size) as i64;
        let limit = page_size as i64;
        
        self.query_builder
            .push(" LIMIT ")
            .push_bind(VAL::from(limit))
            .push(" OFFSET ")
            .push_bind(VAL::from(offset));
        
        Ok(self.query_builder)
    }

    /// Cursor pagination query (only supports single primary key), filter conditions can be mixed with where_
    /// 
    /// # Arguments
    /// * `primary_key` - Primary key column name
    /// * `sort_order` - Sort order direction
    /// * `current_cursor` - Current cursor value
    /// * `limit` - Maximum number of records to return
    /// 
    /// # Type Parameters
    /// * `VAL` - Must also implement `From<i64>` trait
    /// * `C` - Cursor type
    /// 
    /// # Returns
    /// A QueryBuilder with the SELECT query or an Error
    /// 
    /// 游标分页查询（仅支持单主键），过滤条件可以和 where_ 混合
    /// 
    /// # 参数
    /// * `primary_key` - 主键列名
    /// * `sort_order` - 排序方向
    /// * `current_cursor` - 当前游标值
    /// * `limit` - 要返回的最大记录数
    /// 
    /// # 类型参数
    /// * `VAL` - 还必须实现 `From<i64>` trait
    /// * `C` - 游标类型
    /// 
    /// # 返回值
    /// 包含 SELECT 查询的 QueryBuilder 或错误
    pub fn cursor<C>(
        self,
        primary_key: &'a str,
        sort_order: &SortOrder,
        current_cursor: Option<VAL>,
        limit: u64,
    ) -> Result<QueryBuilder<'a, DB>, Error> 
    where 
        VAL: Encode<'a, DB> + Type<DB> + From<i64> + 'a,
    {
        let mut query_builder = self.query_builder;
        if let Some(cursor_value) = current_cursor {
            let operator = if sort_order == &SortOrder::Asc { ">" } else { "<" };
            
            if !self.has_filter {
                query_builder.push(" WHERE ");
            }
            
            query_builder.push(primary_key)
                .push(" ").push(operator)
                .push(" ").push_bind(cursor_value);
        }

        if !self.has_order {
            query_builder
                .push(" ORDER BY ")
                .push(primary_key)
                .push(" ")
                .push(sort_order.as_str());
        }

        query_builder.push(" LIMIT ").push_bind(VAL::from(limit as i64));
        
        Ok(query_builder)
    }

    /// Convert to raw QueryBuilder
    /// 
    /// # Returns
    /// The inner QueryBuilder instance
    /// 
    /// 转换为原始 QueryBuilder
    /// 
    /// # 返回值
    /// 内部的 QueryBuilder 实例
    pub fn inner(self) -> QueryBuilder<'a, DB> {
        self.query_builder
    }
}