use std::marker::PhantomData;

use field_access::FieldAccess;
use sqlx::{Database, Encode, Error, QueryBuilder, Type};

use crate::common::{
    conversion::ValueConvert, error::QueryError, fields::extract_with_bind, filter::push_primary_key_conditions, helper::get_table_name, types::PrimaryKey
};

/// Update query builder
/// 
/// This struct provides functionality to build UPDATE SQL queries.
/// 
/// # Type Parameters
/// * `ET` - Entity type that implements FieldAccess trait
/// * `DB` - Database type that implements sqlx::Database trait
/// * `VAL` - Value type that implements Encode and Type traits
/// 
/// 更新查询构建器
/// 
/// 该结构体提供了构建 UPDATE SQL 查询的功能。
/// 
/// # 类型参数
/// * `ET` - 实现 FieldAccess trait 的实体类型
/// * `DB` - 实现 sqlx::Database trait 的数据库类型
/// * `VAL` - 实现 Encode 和 Type traits 的值类型
pub struct Update<'a, ET, DB, VAL>
where
    DB: Database,
{
    query_builder: QueryBuilder<'a, DB>,
    _phantom: PhantomData<(ET, VAL)>,
}

/// Update operations
/// Creates UPDATE + SET + WHERE queries
/// 
/// 更新操作
/// 创建 UPDATE + SET + WHERE 查询
impl<'a, ET, DB, VAL> Update<'a, ET, DB, VAL>
where
    ET: FieldAccess,
    DB: Database,
    VAL: Encode<'a, DB> + Type<DB> + 'a,
{
    /// Create a basic query
    /// 
    /// # Arguments
    /// * `table_name` - Name of the table to update
    /// 
    /// # Returns
    /// A new Update instance
    /// 
    /// 创建基础查询
    /// 
    /// # 参数
    /// * `table_name` - 要更新的表名
    /// 
    /// # 返回值
    /// 新的 Update 实例
    fn new(table_name: impl Into<String>) -> Self {
        let mut query_builder = QueryBuilder::new("UPDATE ");
        query_builder.push(table_name.into()).push(" SET ");
        
        Self {
            query_builder,
            _phantom: PhantomData,
        }
    }

    /// Create an Update instance with the default table name
    /// 
    /// # Returns
    /// A new Update instance with the default table name
    /// 
    /// 创建使用默认表名的 Update 实例
    /// 
    /// # 返回值
    /// 使用默认表名的新 Update 实例
    pub fn default_table() -> Self {
        Self::new(get_table_name::<ET>())
    }

    /// Create an Update instance with a custom table name, can include alias, between FROM and WHERE
    /// 
    /// # Arguments
    /// * `table_name` - Name of the table to update, can include alias
    /// 
    /// # Returns
    /// A new Update instance with the specified table name
    /// 
    /// 创建使用自定义表名的 Update 实例，可以包含别名，介于 FROM 和 WHERE 之间
    /// 
    /// # 参数
    /// * `table_name` - 要更新的表名，可以包含别名
    /// 
    /// # 返回值
    /// 使用指定表名的新 Update 实例
    pub fn table(table_name: impl Into<String>) -> Self {
        Self::new(table_name)
    }
   
    /// Create a single entity update operation
    /// 
    /// # Arguments
    /// * `model` - Entity model to update
    /// * `primary_key` - Primary key definition
    /// * `skip_non_null` - Whether to skip non-null fields
    /// 
    /// # Type Parameters
    /// * `VAL` - Must also implement ValueConvert, Default traits
    /// 
    /// # Returns
    /// A QueryBuilder with the UPDATE query or an Error
    /// 
    /// 创建单个实体更新操作
    /// 
    /// # 参数
    /// * `model` - 要更新的实体模型
    /// * `primary_key` - 主键定义
    /// * `skip_non_null` - 是否跳过非空字段
    /// 
    /// # 类型参数
    /// * `VAL` - 还必须实现 ValueConvert, Default traits
    /// 
    /// # 返回值
    /// 包含 UPDATE 查询的 QueryBuilder 或错误
    pub fn one(
        model: &'a ET,
        primary_key: &PrimaryKey<'a>,
        skip_non_null: bool,
    ) -> Result<QueryBuilder<'a, DB>, Error>
    where
        VAL: Encode<'a, DB> + Type<DB> + ValueConvert + Default + 'a,
    {
        let keys = primary_key.clone();
        let filter_keys = if primary_key.auto_generate() {
            primary_key.get_keys()
        } else {
            vec![]
        };

        let mut query_builder = Self::default_table().query_builder;
        let mut first = true;
        let fields = extract_with_bind::<VAL, _>(
            model.fields(),
            &filter_keys,
            skip_non_null,
            |name, value| {
                if !first {
                    query_builder.push(", ");
                }
                first = false;
                query_builder.push(format!("{} = ", name)).push_bind(value);
            },
        );
        if fields.0.is_empty() {    
            return Err(QueryError::ColumnsListEmpty.into());
        }

        query_builder.push(" WHERE ");
        push_primary_key_conditions::<ET, DB, VAL>(&mut query_builder, model, &keys);

        Ok(query_builder)
    }

    /// Custom SET columns
    /// 
    /// # Arguments
    /// * `set_build_fn` - Function to build the SET part of the query
    /// 
    /// # Returns
    /// The Update instance with the SET part added
    /// 
    /// 自定义 SET 列
    /// 
    /// # 参数
    /// * `set_build_fn` - 构建查询中 SET 部分的函数
    /// 
    /// # 返回值
    /// 添加了 SET 部分的 Update 实例
    pub fn set(
        mut self,
        set_build_fn: impl FnOnce(&mut QueryBuilder<'a, DB>),
    ) -> Self {
        set_build_fn(&mut self.query_builder);
        self
    }

    /// Add WHERE conditions to the query
    /// 
    /// # Arguments
    /// * `filter_build_fn` - Function to build the WHERE conditions
    /// 
    /// # Returns
    /// A QueryBuilder with the UPDATE query or an Error
    /// 
    /// 向查询中添加 WHERE 条件
    /// 
    /// # 参数
    /// * `filter_build_fn` - 构建 WHERE 条件的函数
    /// 
    /// # 返回值
    /// 包含 UPDATE 查询的 QueryBuilder 或错误
    pub fn where_(
        self,
        filter_build_fn: impl FnOnce(&mut QueryBuilder<'_, DB>),
    ) -> Result<QueryBuilder<'a, DB>, Error> {
        let mut query_builder = self.query_builder;
        
        query_builder.push(" WHERE ");
        filter_build_fn(&mut query_builder);

        Ok(query_builder)
    }

    /// Get the inner QueryBuilder
    /// 
    /// # Returns
    /// The inner QueryBuilder instance
    /// 
    /// 获取内部的 QueryBuilder
    /// 
    /// # 返回值
    /// 内部的 QueryBuilder 实例
    pub fn inner(self) -> QueryBuilder<'a, DB> {
        self.query_builder
    }

}