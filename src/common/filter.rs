//! Database query filter utilities
//! 
//! This module provides utility functions for building database query conditions,
//! including primary key handling, conditional bindings, and sorting functionality.
//! These functions are designed to work with the sqlx QueryBuilder to construct
//! safe and efficient database queries.
//! 
//! # 中文
//! 数据库查询过滤器工具
//! 
//! 该模块提供了构建数据库查询条件的实用函数，
//! 包括主键处理、条件绑定功能。
//! 这些函数设计用于与 sqlx QueryBuilder 配合使用，以构建安全高效的数据库查询。

use field_access::FieldAccess;
use sqlx::{Database, Encode, QueryBuilder, Type};

use crate::common::{conversion::ValueConvert, fields::get_value, types::{PrimaryKey}};

/// Push a primary key and value condition binding to the query builder
/// 
/// This function adds WHERE conditions for primary key columns with their corresponding values
/// to a query builder. It handles both single and composite primary keys.
/// 
/// # Type Parameters
/// * `ET` - The entity type that implements FieldAccess trait
/// * `DB` - The database type that implements the Database trait
/// * `VAL` - The value type that implements Encode and Type traits
/// 
/// # Arguments
/// * `qb` - Mutable reference to the QueryBuilder to modify
/// * `primary_key` - The PrimaryKey configuration containing column names
/// * `values` - Vector of values corresponding to the primary key columns
/// 
/// # 中文
/// 推入主键和值的条件绑定到查询构建器
/// 
/// 此函数向查询构建器添加主键列及其对应值的 WHERE 条件。
/// 它处理单个主键和复合主键。
/// 
/// # 类型参数
/// * `ET` - 实现 FieldAccess trait 的实体类型
/// * `DB` - 实现 Database trait 的数据库类型
/// * `VAL` - 实现 Encode 和 Type traits 的值类型
/// 
/// # 参数
/// * `qb` - 要修改的 QueryBuilder 的可变引用
/// * `primary_key` - 包含列名的 PrimaryKey 配置
/// * `values` - 与主键列对应的值向量
pub fn push_primary_key_bind<'a, ET, DB, VAL>(
    qb: &mut QueryBuilder<'a, DB>,        
    primary_key: &PrimaryKey<'a>,
    values: &'a Vec<VAL>,
) where
    ET: FieldAccess,
    DB: Database,
    VAL: Encode<'a, DB> + Type<DB> + 'a,
{
    let keys = primary_key.get_keys();
    for (i, key) in keys.iter().enumerate() {
        if let Some(value) = values.get(i) {
            if i > 0 {
                qb.push(" AND ");
            }

            qb.push(*key)
              .push(" = ")
              .push_bind(value);
        }
    }
}

/// Push entity and primary key conditions to the query builder
/// 
/// This function extracts values from an entity model based on primary key column names
/// and adds WHERE conditions to a query builder. It is useful for updating or deleting
/// records based on an entity instance.
/// 
/// # Type Parameters
/// * `ET` - The entity type that implements FieldAccess trait
/// * `DB` - The database type that implements the Database trait
/// * `VAL` - The value type that implements Encode, Type, ValueConvert, and Default traits
/// 
/// # Arguments
/// * `qb` - Mutable reference to the QueryBuilder to modify
/// * `model` - Reference to the entity model from which to extract values
/// * `primary_key` - The PrimaryKey configuration containing column names
/// 
/// # 中文
/// 将实体和主键条件推送到查询构建器
/// 
/// 此函数根据主键列名从实体模型中提取值，并向查询构建器添加 WHERE 条件。
/// 它适用于基于实体实例更新或删除记录。
/// 
/// # 类型参数
/// * `ET` - 实现 FieldAccess trait 的实体类型
/// * `DB` - 实现 Database trait 的数据库类型
/// * `VAL` - 实现 Encode、Type、ValueConvert 和 Default traits 的值类型
/// 
/// # 参数
/// * `qb` - 要修改的 QueryBuilder 的可变引用
/// * `model` - 从中提取值的实体模型的引用
/// * `primary_key` - 包含列名的 PrimaryKey 配置
pub fn push_primary_key_conditions<'a, ET, DB, VAL>(
    qb: &mut QueryBuilder<'a, DB>,        
    model: &'a ET, 
    primary_key: &PrimaryKey<'a>,    
) where
    ET: FieldAccess,
    DB: Database,
    VAL: Encode<'a, DB> + Type<DB> + ValueConvert + Default + 'a,
{
    let keys = primary_key.get_keys();
    for (i, key) in keys.iter().enumerate() {
        if i > 0 {
            qb.push(" AND ");
        }
        let value = get_value::<ET, VAL>(model, *key);
        qb.push(*key)
          .push(" = ")
          .push_bind(value);
    }
}