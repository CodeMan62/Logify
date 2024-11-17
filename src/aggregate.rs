use std::collections::HashMap;
use chrono::{DateTime, Duration, Utc};
use serde_json::Value;

use crate::parser::{LogEntry, LogLevel};

pub struct LogAggregator<'a> {
    entries: &'a [LogEntry],
}

#[derive(Debug)]
pub struct AggregateStats {
    pub total_entries: usize,
    pub level_counts: HashMap<LogLevel, usize>,
    pub action_counts: HashMap<String, usize>,
    pub source_counts: HashMap<String, usize>,
}

#[derive(Debug)]
pub struct TimeStats {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_hours: f64,
    pub entries_per_hour: f64,
}

impl<'a> LogAggregator<'a> {
    pub fn new(entries: &'a [LogEntry]) -> Self {
        LogAggregator { entries }
    }

    /// Calculate comprehensive statistics for all log entries
    pub fn calculate_stats(&self) -> AggregateStats {
        let mut level_counts: HashMap<LogLevel, usize> = HashMap::new();
        let mut action_counts: HashMap<String, usize> = HashMap::new();
        let mut source_counts: HashMap<String, usize> = HashMap::new();

        for entry in self.entries {
            // Count log levels
            *level_counts.entry(entry.level().clone()).or_insert(0) += 1;
            
            // Count actions
            *action_counts.entry(entry.action().to_string()).or_insert(0) += 1;
            
            // Count sources if present
            if let Some(source) = entry.source() {
                *source_counts.entry(source.clone()).or_insert(0) += 1;
            }
        }

        AggregateStats {
            total_entries: self.entries.len(),
            level_counts,
            action_counts,
            source_counts,
        }
    }

    /// Calculate time-based statistics
    pub fn calculate_time_stats(&self) -> Option<TimeStats> {
        if self.entries.is_empty() {
            return None;
        }

        let start_time = self.entries.iter()
            .map(|e| e.timestamp())
            .min()
            .copied()?;

        let end_time = self.entries.iter()
            .map(|e| e.timestamp())
            .max()
            .copied()?;

        let duration = end_time.signed_duration_since(start_time);
        let duration_hours = duration.num_milliseconds() as f64 / 3_600_000.0;
        let entries_per_hour = if duration_hours > 0.0 {
            self.entries.len() as f64 / duration_hours
        } else {
            self.entries.len() as f64
        };

        Some(TimeStats {
            start_time,
            end_time,
            duration_hours,
            entries_per_hour,
        })
    }

    /// Group entries by custom time windows
    pub fn group_by_window(&self, window_size: Duration) -> Vec<(DateTime<Utc>, Vec<&LogEntry>)> {
        if self.entries.is_empty() {
            return Vec::new();
        }

        let mut result = Vec::new();
        let mut current_window = Vec::new();
        let mut window_start = *self.entries[0].timestamp();
        let mut window_end = window_start + window_size;

        for entry in self.entries {
            if entry.timestamp() > &window_end {
                if !current_window.is_empty() {
                    result.push((window_start, current_window));
                    current_window = Vec::new();
                }
                window_start = window_end;
                window_end = window_start + window_size;
            }
            current_window.push(entry);
        }

        // Push the last window if it contains entries
        if !current_window.is_empty() {
            result.push((window_start, current_window));
        }

        result
    }

    /// Aggregate metadata values for a specific key
    pub fn aggregate_metadata_values(&self, key: &str) -> HashMap<String, usize> {
        let mut value_counts = HashMap::new();

        for entry in self.entries {
            if let Some(metadata) = entry.metadata() {
                if let Some(value) = metadata.get(key) {
                    let value_str = value.to_string();
                    *value_counts.entry(value_str).or_insert(0) += 1;
                }
            }
        }

        value_counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use serde_json::json;

    fn create_test_entry(
        timestamp: DateTime<Utc>,
        level: LogLevel,
        action: &str,
        source: Option<&str>,
        metadata: Option<Value>,
    ) -> LogEntry {
        LogEntry::new(
            timestamp,
            level,
            "Test message".to_string(),
            action.to_string(),
            source.map(String::from),
            metadata,
        )
    }

    #[test]
    fn test_calculate_stats() {
        let entries = vec![
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
                LogLevel::Info,
                "login",
                Some("web"),
                None,
            ),
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 1).unwrap(),
                LogLevel::Error,
                "login",
                Some("web"),
                None,
            ),
        ];

        let aggregator = LogAggregator::new(&entries);
        let stats = aggregator.calculate_stats();

        assert_eq!(stats.total_entries, 2);
        assert_eq!(*stats.level_counts.get(&LogLevel::Info).unwrap(), 1);
        assert_eq!(*stats.level_counts.get(&LogLevel::Error).unwrap(), 1);
        assert_eq!(*stats.action_counts.get("login").unwrap(), 2);
    }

    #[test]
    fn test_calculate_time_stats() {
        let entries = vec![
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
                LogLevel::Info,
                "login",
                None,
                None,
            ),
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 1, 0, 0).unwrap(),
                LogLevel::Info,
                "logout",
                None,
                None,
            ),
        ];

        let aggregator = LogAggregator::new(&entries);
        let stats = aggregator.calculate_time_stats().unwrap();

        assert_eq!(stats.duration_hours, 1.0);
        assert_eq!(stats.entries_per_hour, 2.0);
    }

    #[test]
    fn test_group_by_window() {
        let entries = vec![
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
                LogLevel::Info,
                "action1",
                None,
                None,
            ),
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 30, 0).unwrap(),
                LogLevel::Info,
                "action2",
                None,
                None,
            ),
        ];

        let aggregator = LogAggregator::new(&entries);
        let windows = aggregator.group_by_window(Duration::hours(1));

        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].1.len(), 2);
    }

    #[test]
    fn test_aggregate_metadata_values() {
        let entries = vec![
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
                LogLevel::Info,
                "action1",
                None,
                Some(json!({"status": "success"})),
            ),
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 1).unwrap(),
                LogLevel::Info,
                "action2",
                None,
                Some(json!({"status": "success"})),
            ),
        ];

        let aggregator = LogAggregator::new(&entries);
        let status_counts = aggregator.aggregate_metadata_values("status");

        assert_eq!(*status_counts.get("\"success\"").unwrap(), 2);
    }
}