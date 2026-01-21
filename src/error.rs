use thiserror::Error;

/// Comprehensive error types for the task manager
#[derive(Debug, Error)]
pub enum TaskError {
    #[error("Task with ID '{0}' not found")]
    TaskNotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Date parsing error: {0}")]
    DateParseError(String),

    #[error("File operation failed: {0}")]
    FileOperationError(String),

    #[error("Operation not allowed: {0}")]
    OperationNotAllowed(String),
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, TaskError>;

impl TaskError {
    /// Create a validation error from validation errors
    pub fn from_validation_errors(errors: validator::ValidationErrors) -> Self {
        let messages: Vec<String> = errors
            .field_errors()
            .iter()
            .map(|(field, errs)| {
                let field_msgs: Vec<String> = errs
                    .iter()
                    .map(|err| err.message.as_ref().unwrap_or(&err.code).to_string())
                    .collect();
                format!("{}: {}", field, field_msgs.join(", "))
            })
            .collect();

        TaskError::ValidationError(messages.join("; "))
    }

}