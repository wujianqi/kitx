use std::{iter::once, marker::PhantomData};

use field_access::FieldAccess;
use sqlx::{Database, Encode, Error, QueryBuilder, Type};

use crate::common::{
    conversion::ValueConvert, error::QueryError, fields::batch_extract, helper::get_table_name, types::PrimaryKey
};

/// Insert query builder
/// 
/// This struct provides functionality to build INSERT SQL queries.
/// 
/// # Type Parameters
/// * `ET` - Entity type that implements FieldAccess trait
/// * `DB` - Database type that implements sqlx::Database trait
/// * `VAL` - Value type that implements Encode, Type, and ValueConvert traits
/// 
/// 插入查询构建器
/// 
/// 该结构体提供了构建 INSERT SQL 查询的功能。
/// 
/// # 类型参数
/// * `ET` - 实现 FieldAccess trait 的实体类型
/// * `DB` - 实现 sqlx::Database trait 的数据库类型
/// * `VAL` - 实现 Encode、Type 和 ValueConvert traits 的值类型
pub struct Insert<'a, ET, DB, VAL>
where
    DB: Database,
{
    query_builder: QueryBuilder<'a, DB>,
    _phantom: PhantomData<(ET, VAL)>,
}

impl<'a, ET, DB, VAL> Insert<'a, ET, DB, VAL>
where
    ET: FieldAccess,
    DB: Database,
    VAL: Encode<'a, DB> + Type<DB> + ValueConvert + 'a,
{

    /// Create custom table and columns, between INTO and VALUES
    /// 
    /// # Arguments
    /// * `table_and_columns_build_fn` - Function to build table and columns part of the query
    /// 
    /// # Returns
    /// A new Insert instance
    /// 
    /// 创建自定义表与列，介于 INTO 和 VALUES 之间
    /// 
    /// # 参数
    /// * `table_and_columns_build_fn` - 构建查询中表和列部分的函数
    /// 
    /// # 返回值
    /// 新的 Insert 实例
    pub fn into(table_and_columns_build_fn: impl FnOnce(&mut QueryBuilder<'_, DB>)) -> Self {
        let mut query_builder = QueryBuilder::new("INSERT INTO ");
        table_and_columns_build_fn(&mut query_builder);
   
        Self { 
            query_builder,
            _phantom: PhantomData
        }
    }    

    /// Create multiple records insert operation
    /// 
    /// # Arguments
    /// * `models` - Collection of entity models to insert
    /// * `primary_key` - Primary key definition
    /// 
    /// # Returns
    /// A QueryBuilder with the INSERT query or an Error
    /// 
    /// 创建多条记录插入操作
    /// 
    /// # 参数
    /// * `models` - 要插入的实体模型集合
    /// * `primary_key` - 主键定义
    /// 
    /// # 返回值
    /// 包含 INSERT 查询的 QueryBuilder 或错误
    pub fn many(
        models: impl IntoIterator<Item = &'a ET>, 
        primary_key: &PrimaryKey<'a>
    ) -> Result<QueryBuilder<'a, DB>, Error>
    {
        let models: Vec<_> = models.into_iter().collect();
        if models.is_empty() {
            return Err(QueryError::NoEntitiesProvided.into());
        }

        let keys = if primary_key.auto_generate() {
            primary_key.get_keys()
        } else {
            vec![]
        };
        let (names, values) = batch_extract::<ET, VAL>(&models, &keys, false);
        let tabe_name = get_table_name::<ET>();
        let mut query_builder = QueryBuilder::new(format!("INSERT INTO {} ({}) ", tabe_name, names.join(", ")));

        query_builder.push_values(
            values,
            |mut b, row| {
                for value in row {
                    b.push_bind(value);
                }
            }
        );

        Ok(query_builder)
    }

    /// Create single record insert operation
    /// 
    /// # Arguments
    /// * `model` - Entity model to insert
    /// * `primary_key` - Primary key definition
    /// 
    /// # Returns
    /// A QueryBuilder with the INSERT query or an Error
    /// 
    /// 创建单条记录插入操作
    /// 
    /// # 参数
    /// * `model` - 要插入的实体模型
    /// * `primary_key` - 主键定义
    /// 
    /// # 返回值
    /// 包含 INSERT 查询的 QueryBuilder 或错误
    pub fn one(
        model: &'a ET,
        primary_key: &PrimaryKey<'a>,
    ) -> Result<QueryBuilder<'a, DB>, Error>
    {
        Self::many(once(model), primary_key)
    }

    /// Custom VALUES or value-related query statements
    /// 
    /// # Arguments
    /// * `build_fn` - Function to build values part of the query
    /// 
    /// # Returns
    /// A QueryBuilder with the INSERT query or an Error
    /// 
    /// 自定义 VALUES 或与值相关的查询语句
    /// 
    /// # 参数
    /// * `build_fn` - 构建查询中值部分的函数
    /// 
    /// # 返回值
    /// 包含 INSERT 查询的 QueryBuilder 或错误
    pub fn values(
        self,
        build_fn: impl FnOnce(&mut QueryBuilder<'_, DB>),
    ) -> Result<QueryBuilder<'a, DB>, Error> {
        let mut query_builder = self.query_builder;
        build_fn(&mut query_builder);

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