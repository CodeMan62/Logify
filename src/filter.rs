use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::parser::{LogEntry, LogLevel};

pub struct LogFilter<'a> {
    entries: &'a [LogEntry],
}

impl<'a> LogFilter<'a> {
    pub fn new(entries: &'a [LogEntry]) -> Self {
        LogFilter { entries }
    }

    pub fn by_level(&self, level: LogLevel) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level() == &level)
            .collect()
    }

    pub fn by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| {
                let timestamp = entry.timestamp();
                timestamp >= &start && timestamp <= &end
            })
            .collect()
    }

    pub fn by_action(&self, action: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.action() == action)
            .collect()
    }

    pub fn by_source(&self, source: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.source().as_ref().map_or(false, |s| s == source))
            .collect()
    }

    pub fn with_metadata_key(&self, key: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| {
                entry.metadata().as_ref().map_or(false, |metadata| {
                    metadata.get(key).is_some()
                })
            })
            .collect()
    }

    pub fn custom_filter<F>(&self, predicate: F) -> Vec<&LogEntry>
    where
        F: Fn(&LogEntry) -> bool,
    {
        self.entries
            .iter()
            .filter(|entry| predicate(entry))
            .collect()
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
    fn test_filter_by_level() {
        let entries = vec![
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
                LogLevel::Info,
                "action1",
                None,
                None,
            ),
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
                LogLevel::Error,
                "action2",
                None,
                None,
            ),
        ];

        let filter = LogFilter::new(&entries);
        let info_entries = filter.by_level(LogLevel::Info);
        assert_eq!(info_entries.len(), 1);
        assert_eq!(info_entries[0].action(), "action1");
    }

    #[test]
    fn test_filter_by_time_range() {
        let entries = vec![
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
                LogLevel::Info,
                "action1",
                None,
                None,
            ),
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 2, 0, 0, 0).unwrap(),
                LogLevel::Info,
                "action2",
                None,
                None,
            ),
        ];

        let filter = LogFilter::new(&entries);
        let filtered = filter.by_time_range(
            Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2023, 1, 1, 23, 59, 59).unwrap(),
        );
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].action(), "action1");
    }
}