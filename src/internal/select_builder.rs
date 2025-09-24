use std::marker::PhantomData;

use crate::common::{error::QueryError, filter::push_primary_key_bind, helper::get_table_name, types::{JoinType, PrimaryKey, Order}};
use field_access::FieldAccess;
use sqlx::{Database, Encode, Error, QueryBuilder, Type};

/// Select query builder
/// 
/// This struct provides functionality to build complete SELECT SQL queries
/// with support for all major SQL clauses.
/// 
/// # Type Parameters
/// * `ET` - Entity type that implements FieldAccess and Default traits
/// * `DB` - Database type that implements sqlx::Database trait
/// * `VAL` - Value type that implements Encode and Type traits
/// 
/// 查询构建器
/// 
/// 该结构体提供了构建完整 SELECT SQL 查询的功能，
/// 支持所有主要 SQL 子句。
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
    table_name: String,
    has_from: bool,
    has_filter: bool,
    has_order: bool,
    has_group_by: bool,
    has_having: bool,
    _phantom: PhantomData<(ET, VAL)>,
}


impl<'a, ET, DB, VAL> Select<'a, ET, DB, VAL>
where
    ET: FieldAccess + Default,
    DB: Database,
    VAL: Encode<'a, DB> + Type<DB> + 'a,
{

    pub fn table() -> Self {
        Self::with_table(&get_table_name::<ET>())
    }

    /// 开始构建 SELECT 查询（指定表名）
    pub fn with_table(table_name: impl Into<String>) -> Self {
        Self::from_query_with_table(QueryBuilder::new(""), table_name)
    }

    /// 从外部查询构建器创建 SELECT 构建器（使用默认表名）
    pub fn from_query(qb: QueryBuilder<'a, DB>) -> Self {
        Self::from_query_with_table(qb, &get_table_name::<ET>())
    }

    /// 从外部查询构建器创建 SELECT 构建器（指定表名）
    pub fn from_query_with_table(mut qb: QueryBuilder<'a, DB>, table_name: impl Into<String>) -> Self {
        qb.push("SELECT ");

        Self {
            query_builder: qb,
            table_name: table_name.into(),
            has_from: false,
            has_filter: false,
            has_order: false,
            has_group_by: false,
            has_having: false,
            _phantom: PhantomData,
        }
    }

    /// 添加自定义列
    pub fn columns(
        mut self,
        column_build_fn: impl FnOnce(&mut QueryBuilder<'_, DB>),
    ) -> Self {
        if self.has_from {
            return self;
        }
        
        column_build_fn(&mut self.query_builder);
        self.query_builder.push(" FROM ")
            .push(&self.table_name);

        self.has_from = true;
        self
    }

    /// 添加所有字段
    fn add_from_clause(&mut self) {
        let columns = ET::default().field_names().join(", ");
        self.query_builder.push(columns)
            .push(" FROM ")
            .push(&self.table_name);

        self.has_from = true;
    }

    /// 添加 JOIN 子句
    /// 
    /// # Arguments
    /// * `join_type` - JOIN 类型（INNER, LEFT, RIGHT 等）
    /// * `table` - 要连接的表（可包含别名）
    /// * `on_condition` - ON 条件构建函数
    /// 
    /// # Returns
    /// 添加了 JOIN 的 Select 实例
    pub fn join(
        mut self,
        join_type: JoinType,
        table: impl Into<String>,
        on_condition: impl FnOnce(&mut QueryBuilder<'_, DB>),
    ) -> Self {
        if !self.has_from {
            self.add_from_clause();
        }

        let join_keyword = match join_type {
            JoinType::Inner => "INNER JOIN",
            JoinType::Left => "LEFT JOIN",
            JoinType::Right => "RIGHT JOIN",
            JoinType::Full => "FULL JOIN",
            JoinType::Cross => "CROSS JOIN",
        };
        
        self.query_builder
            .push(" ")
            .push(join_keyword)
            .push(" ")
            .push(table.into())
            .push(" ON ");
        
        on_condition(&mut self.query_builder);
        self
    }

    /// 添加 GROUP BY 子句
    /// 
    /// # Arguments
    /// * `field` - 分组字段（可为表达式）
    /// 
    /// # Returns
    pub fn group_by(mut self, field: impl Into<String>) -> Self {
        if !self.has_from {
            self.add_from_clause();
        }

        let field = field.into();
      
        if self.has_group_by {
            self.query_builder.push(", ").push(&field);
        } else {
            self.query_builder.push(" GROUP BY ").push(&field);
            self.has_group_by = true;
        }
        
        self
    }

    /// 添加 HAVING 子句（必须在 GROUP BY 之后）
    /// 
    /// # Arguments
    /// * `condition` - HAVING 条件构建函数
    /// 
    /// # Returns
    /// 添加了 HAVING 的 Select 实例   
    pub fn having(
        mut self,
        condition: impl FnOnce(&mut QueryBuilder<'_, DB>),
    ) -> Self {
        if !self.has_group_by {
            return self;
        }

        if !self.has_having {
            self.query_builder.push(" HAVING ");
            self.has_having = true;
        }        
        condition(&mut self.query_builder);
        self
    }

    /// 通过主键查询
    /// 
    /// # Arguments
    /// * `primary_key` - 主键定义
    /// * `primary_value` - 主键值
    /// 
    /// # Returns
    /// 添加了主键条件的 Select 实例
    pub fn by_primary_key(mut self, primary_key: &PrimaryKey<'a>, primary_value: &'a Vec<VAL>,) -> Self {
        if !self.has_from {
            self.add_from_clause();
        }
        if !self.has_filter {
            self.query_builder.push(" WHERE ");
            self.has_filter = true;
        } else {
            self.query_builder.push(" AND ");
        }
        push_primary_key_bind::<ET, DB, VAL>(&mut self.query_builder, primary_key, &primary_value);
        self
    }

    /// 添加 WHERE 过滤条件
    /// 
    /// # Arguments
    /// * `filter_build_fn` - 构建过滤条件的函数
    /// 
    /// # Returns
    /// 添加了过滤条件的 Select 实例
    pub fn filter(
        mut self,
        filter_build_fn: impl FnOnce(&mut QueryBuilder<'_, DB>),
    ) -> Self
    {
        if !self.has_from {
            self.add_from_clause();
        }
        if !self.has_filter {
            self.query_builder.push(" WHERE ");
            self.has_filter = true;
        }
        filter_build_fn(&mut self.query_builder);
        self
    }

    /// 添加排序条件
    /// 
    /// # Arguments
    /// * `field` - 排序字段（可为表达式）
    /// * `order` - 排序方向
    /// 
    /// # Returns
    /// 添加了排序的 Select 实例
    pub fn order_by(mut self, field: impl Into<String>, order: Order) -> Self {
        if !self.has_from {
            self.add_from_clause();
        }
        let field = field.into();
        let order_str = order.as_str();

        if !self.has_order {
            self.query_builder.push(" ORDER BY ");
            self.has_order = true;
        } else {
            self.query_builder.push(", ");
        }
        self.query_builder.push(&field)
            .push(" ")
            .push(order_str);
        self
    }

    /// 添加传统分页
    /// 
    /// # Arguments
    /// * `page_number` - 页码（从1开始）
    /// * `page_size` - 每页记录数
    /// 
    /// # Returns
    pub fn paginate(mut self, page_number: u64, page_size: u64) -> Result<QueryBuilder<'a, DB>, Error> 
    where
        VAL: From<i64> + 'a,
    {
        if !self.has_from {
            self.add_from_clause();
        }
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

    /// 添加游标分页
    /// 
    /// # Arguments
    /// * `primary_key` - 主键列名
    /// * `sort_order` - 排序方向
    /// * `current_cursor` - 当前游标值
    /// * `limit` - 返回记录数
    /// 
    /// # Returns
    pub fn cursor(
        mut self, 
        primary_key: &'a str, 
        sort_order: Order, 
        current_cursor: Option<VAL>, 
        limit: u64
    ) -> Result<QueryBuilder<'a, DB>, Error>
    where
        VAL: From<i64> + 'a,
    {
        if !self.has_from {
            self.add_from_clause();
        }
        if limit < 1 {
            return Err(QueryError::PageNumberInvalid.into());
        }
        if let Some(cursor_value) = current_cursor {
            let operator = if sort_order == Order::Asc { ">" } else { "<" };
            
            if !self.has_filter {
                self.query_builder.push(" WHERE ");
                self.has_filter = true;
            } else {
                self.query_builder.push(" AND ");
            }
            
            self.query_builder.push(primary_key)
                .push(" ").push(operator)
                .push(" ").push_bind(cursor_value);
            
        }
        self = self.order_by(primary_key, sort_order);        
        self.query_builder.push(" LIMIT ").push_bind(VAL::from(limit as i64));
        
        Ok(self.query_builder)
    }

    /// 构建最终查询
    /// 
    /// # Returns
    pub fn finish(mut self) -> QueryBuilder<'a, DB> {
        if !self.has_from {
            self.add_from_clause();
        }
        self.query_builder
    }
}