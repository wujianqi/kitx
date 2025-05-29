use std::fmt::Debug;

use sqlx::Error;

use super::error::RelationError;

#[derive(Debug, Clone, Copy)]
pub enum RelationType {
    OneToOne,
    OneToMany,
    ManyToMany,
}

pub struct EntitiesRelation<'a, D> {
    primary_key: &'a D,
    rel_type: RelationType,
}

impl<'a, D> EntitiesRelation<'a, D>
where
    D: PartialEq + Debug + 'a,
{
    fn new(rel_type: RelationType, primary_key: &'a D) -> Self {
        Self { rel_type,  primary_key}
    }

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

    pub fn one_to_one(primary_key: &'a D) -> Self {
        Self::new(RelationType::OneToOne, primary_key)
    }
    
    pub fn one_to_many(primary_key: &'a D) -> Self {
        Self::new(RelationType::OneToMany, primary_key)
    }
    
    pub fn many_to_many(primary_key: &'a D) -> Self {
        Self::new(RelationType::ManyToMany, primary_key)
    }
}
