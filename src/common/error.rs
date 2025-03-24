use sqlx::error::{DatabaseError, ErrorKind};
use std::{error::Error, fmt};
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

impl OperationError {

    /// Creates a new OperationError instance.
    /// # Arguments
    /// * `message` - The error message.
    pub fn new(message: String) -> SqlxError {
        SqlxError::Database(Box::new(OperationError { message }))
    }
}
