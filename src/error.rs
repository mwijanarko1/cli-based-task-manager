use std::fmt;
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

    #[error("Configuration error: {0}")]
    ConfigError(#[from] config::ConfigError),

    #[error("Invalid priority: {0}")]
    InvalidPriority(String),

    #[error("Invalid status: {0}")]
    InvalidStatus(String),

    #[error("File operation failed: {0}")]
    FileOperationError(String),

    #[error("Database operation failed: {0}")]
    DatabaseError(String),

    #[error("Task already exists with ID: {0}")]
    TaskAlreadyExists(String),

    #[error("Operation not allowed: {0}")]
    OperationNotAllowed(String),
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, TaskError>;

/// Validation result for input validation
pub type ValidationResult<T> = std::result::Result<T, validator::ValidationErrors>;

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

    /// Check if this is a not found error
    pub fn is_not_found(&self) -> bool {
        matches!(self, TaskError::TaskNotFound(_))
    }

    /// Check if this is a validation error
    pub fn is_validation_error(&self) -> bool {
        matches!(self, TaskError::ValidationError(_))
    }

    /// Get error category for logging
    pub fn category(&self) -> &'static str {
        match self {
            TaskError::TaskNotFound(_) => "not_found",
            TaskError::ValidationError(_) => "validation",
            TaskError::IoError(_) => "io",
            TaskError::JsonError(_) => "serialization",
            TaskError::DateParseError(_) => "parsing",
            TaskError::ConfigError(_) => "configuration",
            TaskError::InvalidPriority(_) => "validation",
            TaskError::InvalidStatus(_) => "validation",
            TaskError::FileOperationError(_) => "io",
            TaskError::DatabaseError(_) => "database",
            TaskError::TaskAlreadyExists(_) => "conflict",
            TaskError::OperationNotAllowed(_) => "authorization",
        }
    }
}