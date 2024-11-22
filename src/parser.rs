use crate::error::LogifyError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

pub struct LogParser;

impl LogParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_file(&self, path: &Path) -> Result<Vec<LogEntry>, LogifyError> {
        let content = std::fs::read_to_string(path)?;
        let entries: Vec<LogEntry> = serde_json::from_str(&content)?;
        Ok(entries)
    }
}
