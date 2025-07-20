use std::fmt::{Display, Formatter, Result, Debug};
use std::error::Error;
#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
use sqlx::error::{DatabaseError, ErrorKind};
#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
use sqlx::Error as SqlxError;

#[derive(Debug)]
pub struct KitxError {
    message: String,
}

#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
#[derive(Debug)]
pub enum QueryError {
    DBPoolNotInitialized,
    NoPrimaryKeyDefined,
    SingleKeyTypeInvalid,
    CompositeKeyTypeInvalid,
    PageNumberInvalid,
    LimitInvalid,
    KeysListEmpty,    
    ColumnsListEmpty,
    NoEntitiesProvided,
    ValueInvalid(String),
    NoValuesProvided,
    PrimaryKeyNotFound(String),
    Other(String),
    SoftDeleteNotEnabled,
}

#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
#[derive(Debug)]
pub enum SoftDeleteError {
    NoTableNameDefined,
    SoftDeleteConfigNotSet,
    SoftDeleteColumnTypeInvalid,
    RestoreOperationNotSupported,
}

#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
#[derive(Debug)]
pub enum RelationError {
    ValueEmpty(usize),
    ValueMismatch(usize, String, String),
}

#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
impl QueryError {
    pub fn message(&self) -> String {
        match self {
            Self::DBPoolNotInitialized => "Database pool not initialized".to_string(),
            Self::NoPrimaryKeyDefined => "No primary key defined".to_string(),
            Self::PageNumberInvalid => "Page number and page size must be greater than 0".to_string(),
            Self::SingleKeyTypeInvalid => "Primary key type must be a single value".to_string(),
            Self::CompositeKeyTypeInvalid => "Primary key type must be a vector".to_string(),
            Self::LimitInvalid => "Limit must be greater than 0".to_string(),
            Self::KeysListEmpty => "Keys list cannot be empty".to_string(),
            Self::ValueInvalid(column_name) => format!("Field {} has an invalid value", column_name),
            Self::ColumnsListEmpty => "No valid fields provided".to_string(),
            Self::NoEntitiesProvided => "No entities provided".to_string(),
            Self::PrimaryKeyNotFound(key_name) => format!("Primary key {} not found", key_name),
            Self::NoValuesProvided => "No values provided".to_string(),
            Self::Other(msg) => msg.to_owned(),
            Self::SoftDeleteNotEnabled => "Soft delete is not enabled for this table".to_string(),
        }
    }
}


#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
impl SoftDeleteError {
    pub fn message(&self) -> String {
        match self {
            Self::NoTableNameDefined => "Table is excluded from soft delete".to_string(),
            Self::SoftDeleteConfigNotSet => "Soft delete configuration not found".to_string(),
            Self::SoftDeleteColumnTypeInvalid=> "Soft delete column type must be boolean".to_string(),
            Self::RestoreOperationNotSupported => "Restore operation not supported without soft delete configuration or valid primary key".to_string(),
        }
    }
}

#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
impl RelationError {
    pub fn message(&self) -> String {
        match self {
            Self::ValueEmpty(size) => format!("Expected non-empty values, got {}", size),
            Self::ValueMismatch(index, expected, actual) => 
                format!("Value mismatch: index {}, expected {}, got {}", index, expected, actual),
        }
    }
}

impl Display for KitxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?}", self.message)
    }
}

impl Error for KitxError {}

impl KitxError { 
    /// Creates a new KitxError instance
    /// 
    /// # Arguments
    /// * `message` - Error description message
    pub fn new(message: String) -> Self {
        KitxError { message }
    }
}

#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
impl From<QueryError> for SqlxError {
    fn from(err: QueryError) -> Self {
        SqlxError::Database(Box::new(KitxError {  message: err.message() }))
    }
}
#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
impl From<SoftDeleteError> for SqlxError {
    fn from(err: SoftDeleteError) -> Self {
        SqlxError::Database(Box::new(KitxError {  message: err.message() }))
    }
}
#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
impl From<RelationError> for SqlxError {
    fn from(err: RelationError) -> Self {
        SqlxError::Database(Box::new(KitxError { message: err.message() }))
    }
}


#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
impl DatabaseError for KitxError {
    fn as_error(&self) -> &(dyn Error + Send + Sync + 'static) {
        self
    }
    fn message(&self) -> &str {
        &self.message
    }
    fn as_error_mut(&mut self) -> &mut (dyn Error + Send + Sync + 'static) {
        self
    }

    fn into_error(self: Box<Self>) -> Box<dyn Error + Send + Sync + 'static> {
        self
    }

    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}