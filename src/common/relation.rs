use std::fmt::Debug;

use sqlx::Error;

use super::error::RelationError;

#[derive(Debug, Clone, Copy)]
pub enum RelationType {
    OneToOne,
    OneToMany,
    ManyToMany,
}

/// Represents a relation between entities in a database.
pub struct EntitiesRelation<'a, D> {
    primary_key: &'a D,
    rel_type: RelationType,
}

/// Implementation of the EntitiesRelation struct.
impl<'a, D> EntitiesRelation<'a, D>
where
    D: PartialEq + Debug + 'a,
{
    /// Creates a new EntitiesRelation instance.
    /// # Arguments
    /// * `rel_type` - The type of relation (OneToOne, OneTo Many, ManyToMany).
    /// * `primary_key` - The primary key of the entity.
    /// Returns a new EntitiesRelation instance.
    fn new(rel_type: RelationType, primary_key: &'a D) -> Self {
        Self { rel_type,  primary_key}
    }

    /// Validates the relation between entities.
    /// It checks whether the number of values matches the relation type and whether the values match the primary key.
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
    /// This relation type allows a single entity to be associated with another single entity.
    pub fn one_to_one(primary_key: &'a D) -> Self {
        Self::new(RelationType::OneToOne, primary_key)
    }
    
    /// Creates a new EntitiesRelation instance for one-to-many relations.
    /// This relation type allows one entity to be associated with multiple other entities.
    pub fn one_to_many(primary_key: &'a D) -> Self {
        Self::new(RelationType::OneToMany, primary_key)
    }
    
    /// Creates a new EntitiesRelation instance for many-to-many relations.
    /// This relation type allows multiple entities to be associated with multiple other entities.
    pub fn many_to_many(primary_key: &'a D) -> Self {
        Self::new(RelationType::ManyToMany, primary_key)
    }
}
