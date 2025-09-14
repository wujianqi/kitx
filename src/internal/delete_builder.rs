use std::marker::PhantomData;

use field_access::FieldAccess;
use sqlx::{Database, Encode, Error, QueryBuilder, Type};

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
/// # 中文
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
    _phantom: PhantomData<(ET, VAL)>,
}

impl<'a, ET, DB, VAL> Delete<'a, ET, DB, VAL>
where
    ET: FieldAccess,
    DB: Database,
    VAL: Encode<'a, DB> + Type<DB>,
{
    /// Create a new Delete instance with the specified table name
    /// 
    /// # Arguments
    /// * `table_name` - Name of the table to delete from
    /// 
    /// # Returns
    /// A new Delete instance
    /// 
    /// # 中文
    /// 使用指定的表名创建新的 Delete 实例
    /// 
    /// # 参数
    /// * `table_name` - 要删除的表名
    /// 
    /// # 返回值
    /// 新的 Delete 实例
    fn new(table_name: impl Into<String>) -> Self {
        let mut query_builder =  QueryBuilder::new("DELETE FROM ");
        query_builder.push(table_name.into());

        Self {
            query_builder,
            _phantom: PhantomData,
        }      
    }

    /// Create a Delete instance using the default table name derived from the entity type
    /// 
    /// # Returns
    /// A new Delete instance with the default table name
    /// 
    /// # 中文
    /// 创建使用从实体类型派生的默认表名的 Delete 实例
    /// 
    /// # 返回值
    /// 使用默认表名的新 Delete 实例
    pub fn from_default() -> Self {
        Self::new(get_table_name::<ET>())
    }

    /// Create a Delete instance with a custom table name
    /// 
    /// # Arguments
    /// * `table_name` - Custom table name
    /// 
    /// # Returns
    /// A new Delete instance with the specified table name
    /// 
    /// # 中文
    /// 使用自定义表名创建 Delete 实例
    /// 
    /// # 参数
    /// * `table_name` - 自定义表名
    /// 
    /// # 返回值
    /// 使用指定表名的新 Delete 实例
    pub fn from(table_name: impl Into<String>) -> Self {
        Self::new(table_name)
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
    /// # 中文
    /// 通过主键创建 DELETE 查询
    /// 
    /// # 参数
    /// * `primary_key` - 主键定义
    /// * `primary_value` - 要匹配的主键值
    /// 
    /// # 返回值
    /// 包含 DELETE 查询的 QueryBuilder 或错误
    pub fn by_primary_key(
        primary_key: &PrimaryKey<'a>,
        primary_value: &'a Vec<VAL>,
    ) -> Result<QueryBuilder<'a, DB>, Error>
    {
        let mut query_builder = Self::from_default().query_builder;        
        query_builder.push(" WHERE ");
        push_primary_key_bind::<ET, DB, VAL>(&mut query_builder, primary_key, primary_value);

        Ok(query_builder)
    }

    /// Create a DELETE query with custom WHERE conditions
    /// 
    /// # Arguments
    /// * `filter_build_fn` - Function to build the WHERE conditions
    /// 
    /// # Returns
    /// A QueryBuilder with the DELETE query or an Error
    /// 
    /// # 中文
    /// 创建带有自定义 WHERE 条件的 DELETE 查询
    /// 
    /// # 参数
    /// * `filter_build_fn` - 构建 WHERE 条件的函数
    /// 
    /// # 返回值
    /// 包含 DELETE 查询的 QueryBuilder 或错误
    pub fn where_(
        mut self,
        filter_build_fn: impl FnOnce(&mut QueryBuilder<'_, DB>),
    ) -> Result<QueryBuilder<'a, DB>, Error> {
        self.query_builder.push(" WHERE ");
        filter_build_fn(&mut self.query_builder);
        Ok(self.query_builder)
    }

}