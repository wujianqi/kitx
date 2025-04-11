use std::{error::Error, fmt};
#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
use sqlx::error::{DatabaseError, ErrorKind};
#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
use sqlx::Error as SqlxError;

#[derive(Debug)]
pub struct OperationError {
    message: String,
}

impl fmt::Display for OperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.message)
    }
}

impl Error for OperationError {}

impl OperationError {
       
    /// Creates a new OperationError instance.
    /// # Arguments
    /// * `message` - The error message.
    pub fn new(message: String) -> OperationError {
        OperationError { message }
    }

    #[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
    pub fn db(message: String) -> SqlxError {
        SqlxError::Database(Box::new(OperationError { message }))
    }
}

#[cfg(any(feature = "mysql", feature = "sqlite", feature = "postgres"))]
impl DatabaseError for OperationError {
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