use std::io;
use std::fmt;
use std::result;
use thiserror::Error;
use chrono::ParseError as ChronoParseError;
use crate::parser;
/// Custom result type for Logify operations
pub type Result<T> = result::Result<T, LogifyError>;

#[derive(Error, Debug)]
pub enum LogifyError {
    #[error("Parser error: {0}")]
    Parser(#[from] parser::ParserError),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Failed to parse log file: {0}")]
    ParseError(String),

    #[error("Invalid log format: {0}")]
    FormatError(String),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("CSV error: {0}")]
    CsvError(#[from] csv::Error),

    #[error("Date/time parsing error: {0}")]
    TimeError(#[from] ChronoParseError),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Invalid filter condition: {0}")]
    FilterError(String),

    #[error("Export error: {0}")]
    ExportError(String),

    #[error("Analysis error: {0}")]
    AnalysisError(String),

    #[error("Invalid operation: {0}")]
    OperationError(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}


impl LogifyError {
    /// Returns true if the error is related to I/O operations
    pub fn is_io_error(&self) -> bool {
        matches!(self, LogifyError::Io(_))
    }

    /// Returns true if the error is related to parsing
    pub fn is_parse_error(&self) -> bool {
        matches!(self, LogifyError::ParseError(_))
    }

    /// Returns true if the error is related to validation
    pub fn is_validation_error(&self) -> bool {
        matches!(self, LogifyError::ValidationError(_))
    }

    /// Get a reference to the error message
    pub fn message(&self) -> String {
        self.to_string()
    }

    /// Convert to a user-friendly error message
    pub fn user_friendly_message(&self) -> String {
        match self {
            LogifyError::Parser(err) => format!("Parser error: {}", err),
LogifyError::Io(err) => format!("File operation failed: {}", err),
            LogifyError::ParseError(msg) => format!("Failed to parse log file: {}", msg),
            LogifyError::FormatError(msg) => format!("Invalid log format: {}", msg),
            LogifyError::JsonError(err) => format!("JSON processing failed: {}", err),
            LogifyError::CsvError(err) => format!("CSV processing failed: {}", err),
            LogifyError::TimeError(err) => format!("Invalid date/time format: {}", err),
            LogifyError::ConfigError(msg) => format!("Configuration error: {}", msg),
            LogifyError::FilterError(msg) => format!("Invalid filter: {}", msg),
            LogifyError::ExportError(msg) => format!("Export failed: {}", msg),
            LogifyError::AnalysisError(msg) => format!("Analysis failed: {}", msg),
            LogifyError::OperationError(msg) => format!("Invalid operation: {}", msg),
            LogifyError::MissingField(field) => format!("Missing required field: {}", field),
            LogifyError::ValidationError(msg) => format!("Validation failed: {}", msg),
        }
    }
}

/// Helper macro for creating validation errors
#[macro_export]
macro_rules! validation_error {
    ($msg:expr) => {
        LogifyError::ValidationError($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        LogifyError::ValidationError(format!($fmt, $($arg)*))
    };
}

/// Helper macro for creating parse errors
#[macro_export]
macro_rules! parse_error {
    ($msg:expr) => {
        LogifyError::ParseError($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        LogifyError::ParseError(format!($fmt, $($arg)*))
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error as IoError, ErrorKind};

    #[test]
    fn test_io_error_conversion() {
        let io_error = IoError::new(ErrorKind::NotFound, "file not found");
        let logify_error = LogifyError::from(io_error);
        assert!(logify_error.is_io_error());
    }

    #[test]
    fn test_validation_error_macro() {
        let error = validation_error!("Invalid value: {}", 42);
        assert!(matches!(error, LogifyError::ValidationError(_)));
        assert_eq!(
            error.to_string(),
            "Validation error: Invalid value: 42"
        );
    }

    #[test]
    fn test_parse_error_macro() {
        let error = parse_error!("Failed to parse line {}", 1);
        assert!(matches!(error, LogifyError::ParseError(_)));
        assert_eq!(
            error.to_string(),
            "Failed to parse log file: Failed to parse line 1"
        );
    }

    #[test]
    fn test_user_friendly_message() {
        let error = LogifyError::MissingField("timestamp".to_string());
        assert_eq!(
            error.user_friendly_message(),
            "Missing required field: timestamp"
        );
    }

    #[test]
    fn test_error_categorization() {
        let parse_error = LogifyError::ParseError("test".to_string());
        let validation_error = LogifyError::ValidationError("test".to_string());
        
        assert!(parse_error.is_parse_error());
        assert!(validation_error.is_validation_error());
    }
}