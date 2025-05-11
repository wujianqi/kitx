use std::{error::Error, fmt};
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
pub enum OperationError {
    DBPoolNotInitialized,
    NoPrimaryKeyDefined,
    PageNumberInvalid,
    LimitInvalid,
    NoTableNameDefined,
    SoftDeleteConfigNotSet,
    SoftDeleteColumnTypeInvalid,
    KeysListEmpty,
    RestoreOperationNotSupported,
    ColumnsListEmpty,
    NoEntitiesProvided,
    ValueInvalid(String),
    NoValuesProvided,
    PrimaryKeyNotFound(String),
    Other(String),
}

#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
impl OperationError {
    pub fn message(&self) -> String {
        match self {
            OperationError::DBPoolNotInitialized => "Database pool not initialized".to_string(),
            OperationError::NoPrimaryKeyDefined => "No primary key defined".to_string(),
            
            OperationError::PageNumberInvalid => "Page number and page size must be greater than 0".to_string(),
            OperationError::LimitInvalid => "Limit must be greater than 0".to_string(),
            OperationError::NoTableNameDefined => "Table is excluded from soft delete".to_string(),
            OperationError::SoftDeleteConfigNotSet => "Soft delete configuration not found".to_string(),
            OperationError::SoftDeleteColumnTypeInvalid=> "Soft delete column type must be boolean".to_string(),
            OperationError::KeysListEmpty => "Keys list cannot be empty".to_string(),
            OperationError::RestoreOperationNotSupported => "Restore operation not supported without soft delete configuration or valid primary key".to_string(),
            OperationError::ValueInvalid(column_name) => format!("Field {} has an invalid value", column_name),
            OperationError::ColumnsListEmpty => "No valid fields provided".to_string(),
            OperationError::NoEntitiesProvided => "No entities provided".to_string(),
            OperationError::PrimaryKeyNotFound(key_name) => format!("Primary key {} not found", key_name),
            OperationError::NoValuesProvided => "No values provided".to_string(),
            OperationError::Other(msg) => msg.to_owned(),
        }
    }
}

impl fmt::Display for KitxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
impl From<OperationError> for SqlxError {
    fn from(err: OperationError) -> Self {
        SqlxError::Database(Box::new(KitxError {  message: err.message() }))
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