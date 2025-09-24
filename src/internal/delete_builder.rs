use std::marker::PhantomData;

use field_access::FieldAccess;
use sqlx::{Database, Encode, QueryBuilder, Type};

use crate::common::{
    filter::push_primary_key_bind, helper::get_table_name, types::PrimaryKey
};

/// Delete query builder
/// 
/// This struct provides functionality to build DELETE SQL queries.
/// 
/// # Type Parameters
/// * `ET` - Entity type that implements FieldAccess trait
/// * `DB` - Database type that implements sqlx::Database trait
/// * `VAL` - Value type that implements Encode and Type traits
/// 
/// 删除查询构建器
/// 
/// 该结构体提供了构建 DELETE SQL 查询的功能。
/// 
/// # 类型参数
/// * `ET` - 实现 FieldAccess trait 的实体类型
/// * `DB` - 实现 sqlx::Database trait 的数据库类型
/// * `VAL` - 实现 Encode 和 Type traits 的值类型
pub struct Delete<'a, ET, DB, VAL>
where
    DB: Database,
{
    query_builder: QueryBuilder<'a, DB>,
    has_filter: bool,
    _phantom: PhantomData<(ET, VAL)>,
}

impl<'a, ET, DB, VAL> Delete<'a, ET, DB, VAL>
where
    ET: FieldAccess,
    DB: Database,
    VAL: Encode<'a, DB> + Type<DB>,
{
    /// Create a Delete instance using the default table name derived from the entity type
    /// 
    /// # Returns
    /// A new Delete instance with the default table name
    /// 
    /// 创建使用从实体类型派生的默认表名的 Delete 实例
    /// 
    /// # 返回值
    /// 使用默认表名的新 Delete 实例
    pub fn table() -> Self {
        Self::with_table(get_table_name::<ET>())
    }

    /// Create a Delete instance with a custom table name
    /// 
    /// # Arguments
    /// * `table_name` - Custom table name
    /// 
    /// # Returns
    /// A new Delete instance with the specified table name
    /// 
    /// 使用自定义表名创建 Delete 实例
    /// 
    /// # 参数
    /// * `table_name` - 自定义表名
    /// 
    /// # 返回值
    /// 使用指定表名的新 Delete 实例
    pub fn with_table(table_name: impl Into<String>) -> Self {
        Self::from_query_with_table(QueryBuilder::new(""), table_name)
    }

    /// 从外部查询构建器创建 INSERT 构建器（使用默认表名）
    pub fn from_query(qb: QueryBuilder<'a, DB>) -> Self {
        Self::from_query_with_table(qb, &get_table_name::<ET>())
    }

    /// 从外部查询构建器创建 INSERT 构建器（指定表名）
    pub fn from_query_with_table(mut query_builder: QueryBuilder<'a, DB>, table_name: impl Into<String>) -> Self {
        query_builder.push("DELETE FROM ").push(table_name.into());

        Self {
            query_builder,
            has_filter: false,
            _phantom: PhantomData,
        }
    }
    
    /// Create a DELETE query by primary key
    /// 
    /// # Arguments
    /// * `primary_key` - Primary key definition
    /// * `primary_value` - Primary key values to match
    /// 
    /// # Returns
    /// A QueryBuilder with the DELETE query or an Error
    /// 
    /// 通过主键创建 DELETE 查询
    /// 
    /// # 参数
    /// * `primary_key` - 主键定义
    /// * `primary_value` - 要匹配的主键值
    /// 
    /// # 返回值
    /// 包含 DELETE 查询的 QueryBuilder 或错误
    pub fn by_primary_key(mut self, primary_key: &PrimaryKey<'a>, primary_value: &'a Vec<VAL>,) -> Self {
        if !self.has_filter {
            self.query_builder.push(" WHERE ");
            self.has_filter = true;
        } else {
            self.query_builder.push(" AND ");
        }
        push_primary_key_bind::<ET, DB, VAL>(&mut self.query_builder, primary_key, &primary_value);
        self
    }

    /// Create a DELETE query with custom WHERE conditions
    /// 
    /// # Arguments
    /// * `filter_build_fn` - Function to build the WHERE conditions
    /// 
    /// # Returns
    /// A QueryBuilder with the DELETE query or an Error
    /// 
    /// 创建带有自定义 WHERE 条件的 DELETE 查询
    /// 
    /// # 参数
    /// * `filter_build_fn` - 构建 WHERE 条件的函数
    /// 
    /// # 返回值
    /// 包含 DELETE 查询的 QueryBuilder 或错误
    pub fn filter(
        mut self,
        filter_build_fn: impl FnOnce(&mut QueryBuilder<'a, DB>),
    ) -> Self {
        self.query_builder.push(" WHERE ");
        filter_build_fn(&mut self.query_builder);

        self
    }

    /// 添加 RETURNING 子句
    /// 
    /// # 参数
    /// * `columns` - 要返回的列
    /// 
    /// # 返回值
    /// 更新后的构建器实例
    #[cfg(any(feature = "sqlite" , feature = "postgres"))]
    pub fn returning<I, S>(mut self, columns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.query_builder.push(" RETURNING ");
        
        let cols: Vec<String> = columns.into_iter().map(|s| s.as_ref().to_string()).collect();
        let mut separated = self.query_builder.separated(", ");
        for col in cols {
            separated.push(col);
        }
        
        self
    }

    /// 添加自定义查询部分
    /// 
    /// # 参数
    /// * `build_fn` - 自定义构建函数
    /// 
    /// # 返回值
    /// 更新后的构建器实例
    pub fn custom<F>(mut self, build_fn: F) -> Self
    where
        F: FnOnce(&mut QueryBuilder<'a, DB>),
    {
        build_fn(&mut self.query_builder);
        self
    }

    /// 构建最终的查询
    /// 
    /// # 返回值
    /// QueryBuilder 实例
    pub fn finish(self) -> QueryBuilder<'a, DB> {
        self.query_builder
    }


}