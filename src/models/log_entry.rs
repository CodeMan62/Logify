use chrono::{DateTime, Utc, ParseError as ChronoParseError};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::str::FromStr;
use std::fmt;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub user_id: String,
    pub action: ActionType,
    pub duration: Duration,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum ActionType {
    Login,
    Logout,
    Search,
    View,
    Update,
    Delete,
    Custom(String),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Duration(pub f64);

#[derive(Error, Debug)]
pub enum LogEntryError {
    #[error("Invalid user ID: cannot be empty")]
    EmptyUserId,

    #[error("Invalid duration: must be non-negative")]
    NegativeDuration,

    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}

impl LogEntry {
    pub fn new(
        timestamp: DateTime<Utc>,
        user_id: String,
        action: ActionType,
        duration: Duration,
    ) -> Result<Self, LogEntryError> {
        let entry = Self {
            timestamp,
            user_id,
            action,
            duration,
            metadata: None,
        };

        entry.validate()?;
        Ok(entry)
    }

    pub fn validate(&self) -> Result<(), LogEntryError> {
        // User ID validation
        if self.user_id.trim().is_empty() {
            return Err(LogEntryError::EmptyUserId);
        }

        // Duration validation
        if self.duration.0 < 0.0 {
            return Err(LogEntryError::NegativeDuration);
        }

        Ok(())
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

impl FromStr for LogEntry {
    type Err = LogEntryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // CSV parsing would typically happen in a CSV handler
        // This is a simplified example
        let parts: Vec<&str> = s.split(',').collect();

        if parts.len() < 4 {
            return Err(LogEntryError::ParseError("Insufficient fields".to_string()));
        }

        let timestamp = parts[0].parse::<DateTime<Utc>>()
            .map_err(|e| LogEntryError::InvalidTimestamp(e.to_string()))?;

        let user_id = parts[1].to_string();

        let action = match parts[2] {
            "login" => ActionType::Login,
            "logout" => ActionType::Logout,
            "search" => ActionType::Search,
            custom => ActionType::Custom(custom.to_string()),
        };

        let duration = Duration(parts[3].parse::<f64>()
            .map_err(|_| LogEntryError::NegativeDuration)?);

        Self::new(timestamp, user_id, action, duration)
    }
}

impl fmt::Display for LogEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{},{},{}",
            self.timestamp,
            self.user_id,
            match &self.action {
                ActionType::Custom(s) => s,
                action => format!("{:?}", action).to_lowercase(),
            },
            self.duration.0
        )
    }
}

// Example Usage
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_log_entry_creation() {
        let timestamp = Utc::now();
        let entry = LogEntry::new(
            timestamp,
            "user123".to_string(),
            ActionType::Login,
            Duration(30.5)
        ).unwrap();

        assert_eq!(entry.user_id, "user123");
        assert_eq!(entry.action, ActionType::Login);
        assert_eq!(entry.duration.0, 30.5);
    }

    #[test]
    fn test_log_entry_validation() {
        // Test empty user ID
        let invalid_entry = LogEntry::new(
            Utc::now(),
            "".to_string(),
            ActionType::Login,
            Duration(30.5)
        );
        assert!(invalid_entry.is_err());

        // Test negative duration
        let negative_entry = LogEntry::new(
            Utc::now(),
            "user123".to_string(),
            ActionType::Login,
            Duration(-10.0)
        );
        assert!(negative_entry.is_err());
    }

    #[test]
    fn test_log_entry_metadata() {
        let entry = LogEntry::new(
            Utc::now(),
            "user123".to_string(),
            ActionType::Login,
            Duration(30.5)
        )
        .unwrap()
        .with_metadata(json!({
            "source": "web",
            "browser": "chrome"
        }));

        assert!(entry.metadata.is_some());
    }
}
