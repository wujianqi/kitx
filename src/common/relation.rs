//! Database entity relationship management
//! 
//! This module provides utilities for managing and validating relationships between 
//! database entities. It includes structures and enums for defining relationship types
//! and validating entity relationships according to business rules.
//! 
//! 数据库实体关系管理
//! 
//! 该模块提供了管理和验证数据库实体间关系的工具。
//! 它包括定义关系类型的结构体和枚举，
//! 以及根据业务规则验证实体关系的功能。

use std::fmt::Debug;

use sqlx::Error;

use super::error::RelationError;

#[derive(Debug, Clone)]
pub enum RelationType {
    OneToOne,
    OneToMany,
    ManyToMany,
}

/// Represents the relationship between entities in the database, used for validation.
/// 
/// # Type Parameters
/// * `D` - The data type of the primary key
/// 
/// 表示数据库中的实体之间的关系，用于验证。
/// 
/// # 类型参数
/// * `D` - 主键的数据类型
pub struct EntitiesRelation<'a, D> {
    primary_key: &'a D,
    rel_type: RelationType,
}

/// Implementation of the EntitiesRelation struct.
/// EntitiesRelation结构体关系的实现。
impl<'a, D> EntitiesRelation<'a, D>
where
    D: PartialEq + Debug,
{
    /// Creates a new EntitiesRelation instance.
    /// # Arguments
    /// * `rel_type` - The type of relation (OneToOne, OneTo Many, ManyToMany).
    /// * `primary_key` - The primary key of the entity.
    /// Returns a new EntitiesRelation instance.
    /// 创建一个新的EntitiesRelation实例。
    /// 
    /// # 参数
    /// * `rel_type` - 关系类型（OneToOne, OneToMany, ManyToMany）。
    /// * `primary_key` - 实体的主键。
    /// 
    /// # 返回值
    /// 返回一个新的EntitiesRelation实例。
    fn new(rel_type: RelationType, primary_key: &'a D) -> Self {
        Self { rel_type,  primary_key}
    }


    /// Validates the relation between entities.
    /// 
    /// It checks whether the number of values matches the relation type and whether the values match the primary key.
    /// 
    /// # Arguments
    /// * `values` - A vector of references to values to validate
    /// 
    /// # Returns
    /// * [Ok] if validation passes
    /// * [Err] with a detailed error if validation fails
    /// 
    /// 验证实体之间的关系。
    /// 
    /// 检查值的数量是否与关系类型匹配，以及值是否与主键匹配。
    /// 
    /// # 参数
    /// * `values` - 需要验证的值的引用向量
    /// 
    /// # 返回值
    /// * 如果验证通过则返回[Ok]
    /// * 如果验证失败则返回包含详细错误信息的[Err]
    pub fn validate(&self, values: Vec<&'a D>) -> Result<(), Error>
    {
        match self.rel_type {
            RelationType::OneToOne => {
                if values.len() != 1 {
                    return Err(RelationError::ValueEmpty(values.len()).into());
                }
                if values[0] != *&self.primary_key {
                    return Err(RelationError::ValueMismatch(0,
                        format!("{:?}", &self.primary_key), 
                        format!("{:?}", &values[0])).into());
                }
            }
            
            RelationType::OneToMany => {
                if values.is_empty() {
                    return Err(RelationError::ValueEmpty(values.len()).into());
                }
                for (i, value) in values.iter().enumerate() {
                    if value != &self.primary_key {
                        return Err(RelationError::ValueMismatch(i,
                            format!("{:?}", &self.primary_key), 
                            format!("{:?}", value)).into());
                    }
                }
            }
            
            RelationType::ManyToMany => {
                if values.is_empty() {
                    return Err(RelationError::ValueEmpty(values.len()).into());
                }
                for (i, value) in values.iter().enumerate() {
                    if value != &self.primary_key {
                        return Err(RelationError::ValueMismatch(i,
                            format!("{:?}", &self.primary_key), 
                            format!("{:?}", value)).into());
                    }
                }
            }
        }
        Ok(())
    }

    /// Creates a new EntitiesRelation instance for one-to-one relations.
    /// 
    /// This relation type allows a single entity to be associated with another single entity.
    /// 
    /// # Arguments
    /// * `primary_key` - The primary key of the entity
    /// 
    /// # Returns
    /// A new EntitiesRelation instance with OneToOne relationship type
    /// 
    /// 创建一对一关系的EntitiesRelation实例。
    /// 
    /// 此关系类型允许单个实体与另一个单个实体关联。
    /// 
    /// # 参数
    /// * `primary_key` - 实体的主键
    /// 
    /// # 返回值
    /// 具有一对一关系类型的新EntitiesRelation实例
    pub fn one_to_one(primary_key: &'a D) -> Self {
        Self::new(RelationType::OneToOne, primary_key)
    }
    
    /// Creates a new EntitiesRelation instance for one-to-many relations.
    /// 
    /// This relation type allows one entity to be associated with multiple other entities.
    /// 
    /// # Arguments
    /// * `primary_key` - The primary key of the entity
    /// 
    /// # Returns
    /// A new EntitiesRelation instance with OneToMany relationship type
    /// 
    /// 创建一对多关系的EntitiesRelation实例。
    /// 
    /// 此关系类型允许一个实体与多个其他实体关联。
    /// 
    /// # 参数
    /// * `primary_key` - 实体的主键
    /// 
    /// # 返回值
    /// 具有一对多关系类型的新EntitiesRelation实例
    pub fn one_to_many(primary_key: &'a D) -> Self {
        Self::new(RelationType::OneToMany, primary_key)
    }
    
    /// Creates a new EntitiesRelation instance for many-to-many relations.
    /// 
    /// This relation type allows multiple entities to be associated with multiple other entities.
    /// 
    /// # Arguments
    /// * `primary_key` - The primary key of the entity
    /// 
    /// # Returns
    /// A new EntitiesRelation instance with ManyToMany relationship type
    /// 
    /// 创建多对多关系的EntitiesRelation实例。
    /// 
    /// 此关系类型允许多个实体与多个其他实体关联。
    /// 
    /// # 参数
    /// * `primary_key` - 实体的主键
    /// 
    /// # 返回值
    /// 具有多对多关系类型的新EntitiesRelation实例
    pub fn many_to_many(primary_key: &'a D) -> Self {
        Self::new(RelationType::ManyToMany, primary_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_to_one_relation() {
        let relation = EntitiesRelation::one_to_one(&1);
        let values = vec![&1];
        assert!(relation.validate(values).is_ok());

        let values = vec![&2];
        assert!(relation.validate(values).is_err());
    }
}