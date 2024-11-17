use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::parser::{LogEntry, LogLevel};

pub struct LogTransformer<'a> {
    entries: &'a [LogEntry],
}

#[derive(Debug)]
pub struct TransformedEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub action: String,
    pub source: Option<String>,
    pub metadata: Option<Value>,
    pub enriched_data: HashMap<String, Value>,
}

impl<'a> LogTransformer<'a> {
    pub fn new(entries: &'a [LogEntry]) -> Self {
        LogTransformer { entries }
    }

    /// Map entries to a different type using the provided transformation function
    pub fn map<T, F>(&self, transform: F) -> Vec<T>
    where
        F: Fn(&LogEntry) -> T,
    {
        self.entries.iter().map(transform).collect()
    }

    /// Enrich entries with additional metadata
    pub fn enrich<F>(&self, enricher: F) -> Vec<TransformedEntry>
    where
        F: Fn(&LogEntry) -> HashMap<String, Value>,
    {
        self.entries
            .iter()
            .map(|entry| {
                let enriched_data = enricher(entry);
                TransformedEntry {
                    timestamp: *entry.timestamp(),
                    level: entry.level().clone(),
                    message: entry.message().to_string(),
                    action: entry.action().to_string(),
                    source: entry.source().clone(),
                    metadata: entry.metadata().clone(),
                    enriched_data,
                }
            })
            .collect()
    }

    /// Extract specific fields from metadata into a flattened structure
    pub fn flatten_metadata(&self, keys: &[&str]) -> Vec<HashMap<String, Value>> {
        self.entries
            .iter()
            .map(|entry| {
                let mut flattened = HashMap::new();
                if let Some(metadata) = entry.metadata() {
                    for &key in keys {
                        if let Some(value) = metadata.get(key) {
                            flattened.insert(key.to_string(), value.clone());
                        }
                    }
                }
                flattened
            })
            .collect()
    }

    /// Transform entries by applying custom rules based on log level
    pub fn transform_by_level<F>(&self, transformer: F) -> Vec<LogEntry>
    where
        F: Fn(&LogEntry, &LogLevel) -> Option<LogEntry>,
    {
        self.entries
            .iter()
            .filter_map(|entry| transformer(entry, entry.level()))
            .collect()
    }

    /// Chain multiple transformations together
    pub fn chain_transforms<T>(
        &self,
        transforms: Vec<Box<dyn Fn(&LogEntry) -> Option<T>>>,
    ) -> Vec<T> {
        self.entries
            .iter()
            .filter_map(|entry| {
                let mut result = None;
                for transform in &transforms {
                    result = transform(entry);
                    if result.is_some() {
                        break;
                    }
                }
                result
            })
            .collect()
    }

    /// Extract and structure error information from error-level logs
    pub fn extract_error_info(&self) -> Vec<HashMap<String, String>> {
        self.entries
            .iter()
            .filter(|entry| entry.level() == &LogLevel::Error)
            .map(|entry| {
                let mut info = HashMap::new();
                info.insert("timestamp".to_string(), entry.timestamp().to_string());
                info.insert("message".to_string(), entry.message().to_string());
                
                if let Some(metadata) = entry.metadata() {
                    if let Some(error_code) = metadata.get("error_code") {
                        info.insert("error_code".to_string(), error_code.to_string());
                    }
                    if let Some(stack_trace) = metadata.get("stack_trace") {
                        info.insert("stack_trace".to_string(), stack_trace.to_string());
                    }
                }
                
                info
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn create_test_entry(
        timestamp: DateTime<Utc>,
        level: LogLevel,
        message: &str,
        metadata: Option<Value>,
    ) -> LogEntry {
        LogEntry::new(
            timestamp,
            level,
            message.to_string(),
            "test_action".to_string(),
            Some("test_source".to_string()),
            metadata,
        )
    }

    #[test]
    fn test_map_transformation() {
        let entries = vec![
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
                LogLevel::Info,
                "Test message",
                None,
            ),
        ];

        let transformer = LogTransformer::new(&entries);
        let messages: Vec<String> = transformer.map(|entry| entry.message().to_string());
        
        assert_eq!(messages, vec!["Test message"]);
    }

    #[test]
    fn test_enrich() {
        let entries = vec![
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
                LogLevel::Info,
                "Test message",
                Some(json!({"user_id": "123"})),
            ),
        ];

        let transformer = LogTransformer::new(&entries);
        let enriched = transformer.enrich(|entry| {
            let mut data = HashMap::new();
            if let Some(metadata) = entry.metadata() {
                if let Some(user_id) = metadata.get("user_id") {
                    data.insert("user_info".to_string(), json!({
                        "id": user_id,
                        "type": "standard"
                    }));
                }
            }
            data
        });

        assert!(!enriched[0].enriched_data.is_empty());
    }

    #[test]
    fn test_flatten_metadata() {
        let entries = vec![
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
                LogLevel::Info,
                "Test message",
                Some(json!({
                    "user_id": "123",
                    "session_id": "abc"
                })),
            ),
        ];

        let transformer = LogTransformer::new(&entries);
        let flattened = transformer.flatten_metadata(&["user_id", "session_id"]);
        
        assert_eq!(flattened[0].len(), 2);
    }

    #[test]
    fn test_extract_error_info() {
        let entries = vec![
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
                LogLevel::Error,
                "Error message",
                Some(json!({
                    "error_code": "E123",
                    "stack_trace": "Stack trace details"
                })),
            ),
        ];

        let transformer = LogTransformer::new(&entries);
        let error_info = transformer.extract_error_info();
        
        assert_eq!(error_info.len(), 1);
        assert!(error_info[0].contains_key("error_code"));
    }
}