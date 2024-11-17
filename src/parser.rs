use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use thiserror::Error;
use csv;
use serde_json;
use std::collections::HashMap;
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LogEntry {
    timestamp: DateTime<Utc>,
    level: LogLevel,
    message: String,
    action: String,
    source: Option<String>,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Hash, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("CSV parsing error: {0}")]
    Csv(#[from] csv::Error),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),
}

impl LogEntry {
    pub fn new(
        timestamp: DateTime<Utc>,
        level: LogLevel,
        message: String,
        action: String,
        source: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> Self {
        LogEntry {
            timestamp,
            level,
            message,
            action,
            source,
            metadata,
        }
    }

    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }

    pub fn level(&self) -> &LogLevel {
        &self.level
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn action(&self) -> &str {
        &self.action
    }
    pub fn metadata(&self) -> &Option<serde_json::Value> {
        &self.metadata
    }
    pub fn source(&self) -> &Option<String> {
        &self.source
    }
    
}

pub fn parse_log_file(path: &Path) -> Result<Vec<LogEntry>, ParserError> {
    let file = File::open(path)?;
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| ParserError::UnsupportedFormat("No file extension".to_string()))?;

    match extension {
        "csv" => parse_csv(file),
        "json" => parse_json(file),
        ext => Err(ParserError::UnsupportedFormat(ext.to_string())),
    }
}

fn parse_csv(file: File) -> Result<Vec<LogEntry>, ParserError> {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(file);
    
    let mut entries = Vec::new();

    for result in reader.deserialize() {
        let entry: LogEntry = result?;
        entries.push(entry);
    }

    Ok(entries)
}

fn parse_json(file: File) -> Result<Vec<LogEntry>, ParserError> {
    let reader = BufReader::new(file);
    let entries: Vec<LogEntry> = serde_json::from_reader(reader)?;
    
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use std::fs::write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_csv() {
        let csv_content = "\
timestamp,level,message,action,source,metadata
2023-01-01T00:00:00Z,info,Test message,test_action,test_source,{\"key\":\"value\"}";
        
        let temp_file = NamedTempFile::new().unwrap();
        write(temp_file.path(), csv_content).unwrap();
        
        let entries = parse_log_file(temp_file.path()).unwrap();
        assert_eq!(entries.len(), 1);
        
        let entry = &entries[0];
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.message, "Test message");
        assert_eq!(entry.action, "test_action");
    }

    #[test]
    fn test_parse_json() {
        let json_content = r#"[{
            "timestamp": "2023-01-01T00:00:00Z",
            "level": "info",
            "message": "Test message",
            "action": "test_action",
            "source": "test_source",
            "metadata": {"key": "value"}
        }]"#;
        
        let temp_file = NamedTempFile::new().unwrap();
        write(temp_file.path(), json_content).unwrap();
        
        let entries = parse_log_file(temp_file.path()).unwrap();
        assert_eq!(entries.len(), 1);
        
        let entry = &entries[0];
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.message, "Test message");
        assert_eq!(entry.action, "test_action");
    }
}

