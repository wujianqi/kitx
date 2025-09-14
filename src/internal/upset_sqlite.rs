use std::{iter::once, marker::PhantomData};

use field_access::FieldAccess;
use sqlx::{Database, Encode, Error, QueryBuilder, Type};

use crate::common::{
    conversion::ValueConvert, error::QueryError, fields::batch_extract, helper::get_table_name, types::PrimaryKey
};

/// SQLite Upsert query builder
/// 
/// This struct provides functionality to build SQLite-specific UPSERT (INSERT ... ON CONFLICT) SQL queries.
/// 
/// # Type Parameters
/// * `ET` - Entity type that implements FieldAccess trait
/// * `DB` - Database type that implements sqlx::Database trait
/// * `VAL` - Value type that implements Encode, Type, and ValueConvert traits
/// 
/// SQLite 更新插入查询构建器
/// 
/// 该结构体提供了构建 SQLite 特定的 UPSERT (INSERT ... ON CONFLICT) SQL 查询的功能。
/// 
/// # 类型参数
/// * `ET` - 实现 FieldAccess trait 的实体类型
/// * `DB` - 实现 sqlx::Database trait 的数据库类型
/// * `VAL` - 实现 Encode、Type 和 ValueConvert traits 的值类型
pub struct Upset<'a, ET, DB, VAL>
where
    ET: FieldAccess,
    DB: Database,
    VAL: Encode<'a, DB> + Type<DB> + ValueConvert + 'a,
{
    _phantom: PhantomData<(&'a ET, DB, VAL)>,
}

impl<'a, ET, DB, VAL> Upset<'a, ET, DB, VAL>
where
    ET: FieldAccess,
    DB: Database,
    VAL: Encode<'a, DB> + Type<DB> + ValueConvert + 'a,
{
    /// 批量执行 UPSERT 操作
    /// 
    /// Batch execute UPSERT operations
    /// 
    /// # Arguments
    /// * `models` - Collection of entity models to upsert
    /// * `primary_key` - Primary key definition
    /// 
    /// # Returns
    /// A QueryBuilder with the UPSERT query or an Error
    /// 
    /// 批量执行 UPSERT 操作
    /// 
    /// # 参数
    /// * `models` - 要更新插入的实体模型集合
    /// * `primary_key` - 主键定义
    /// 
    /// # 返回值
    /// 包含 UPSERT 查询的 QueryBuilder 或错误
    pub fn many(
        models: impl IntoIterator<Item = &'a ET>,
        primary_key: &PrimaryKey<'a>,
    ) -> Result<QueryBuilder<'a, DB>, Error> {
       
        let models: Vec<_> = models.into_iter().collect();
        if models.is_empty() {
            return Err(QueryError::NoEntitiesProvided.into());
        }
        
        let (names, values) = batch_extract::<ET, VAL>(&models, &[], false);
        let keys = primary_key.get_keys();
        let table_name = get_table_name::<ET>();
        
        let mut query_builder = QueryBuilder::new(
            format!("INSERT INTO {} ({}) ", table_name, names.join(", "))
        );

        query_builder.push_values(
            values,
            | mut b, row| {
                for (i, value) in row.into_iter().enumerate() {
                    if keys.contains(&names[i]) && VAL::is_default_value(&value) {
                        b.push(" NULL ");
                    } else {
                        b.push_bind(value);
                    }
                }
            }
        );
        
        if !keys.is_empty() {
            query_builder.push(" ON CONFLICT (")
                    .push(keys.join(", "))
                    .push(") DO UPDATE SET ");

            let mut first = true;
            for name in &names {
                if !first {
                    query_builder.push(", ");
                }
                first = false;
                query_builder.push(format!("{} = EXCLUDED.{}", name, name));
            }
        }

        Ok(query_builder)
    }

    /// Create single record upsert operation
    /// 
    /// # Arguments
    /// * `model` - Entity model to upsert
    /// * `primary_key` - Primary key definition
    /// 
    /// # Returns
    /// A QueryBuilder with the UPSERT query or an Error
    /// 
    /// 创建单条记录更新插入操作
    /// 
    /// # 参数
    /// * `model` - 要更新插入的实体模型
    /// * `primary_key` - 主键定义
    /// 
    /// # 返回值
    /// 包含 UPSERT 查询的 QueryBuilder 或错误
    pub fn one(
        model: &'a ET,
        primary_key: &PrimaryKey<'a>,
    ) -> Result<QueryBuilder<'a, DB>, Error>
    {
        Self::many(once(model), primary_key)
    }
}