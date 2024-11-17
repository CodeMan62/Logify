use std::collections::HashMap;
use chrono::{DateTime, Duration, Utc};

use crate::parser::{LogEntry, LogLevel};

pub struct LogCombiner<'a> {
    primary_entries: &'a [LogEntry],
    secondary_entries: &'a [LogEntry],
}

#[derive(Debug)]
pub struct CombinedEntry<'a> {
    pub primary: &'a LogEntry,
    pub secondary: Option<&'a LogEntry>,
}

impl<'a> LogCombiner<'a> {
    pub fn new(primary_entries: &'a [LogEntry], secondary_entries: &'a [LogEntry]) -> Self {
        LogCombiner {
            primary_entries,
            secondary_entries,
        }
    }

    /// Combine entries based on matching timestamps within a specified tolerance
    pub fn combine_by_timestamp(&self, tolerance: Duration) -> Vec<CombinedEntry<'a>> {
        let mut result = Vec::new();
        let mut secondary_map: HashMap<DateTime<Utc>, &LogEntry> = self
            .secondary_entries
            .iter()
            .map(|entry| (*entry.timestamp(), entry))
            .collect();

        for primary_entry in self.primary_entries {
            let primary_time = primary_entry.timestamp();
            let matching_secondary = secondary_map.iter()
                .find(|(time, _)| {
                    let diff = if **time > *primary_time {
                        **time - *primary_time
                    } else {
                        *primary_time - **time
                    };
                    diff <= tolerance
                })
                .map(|(_, entry)| *entry);

            result.push(CombinedEntry {
                primary: primary_entry,
                secondary: matching_secondary,
            });
        }

        result
    }

    /// Combine entries based on a matching key in their metadata
    pub fn combine_by_metadata_key(&self, key: &str) -> Vec<CombinedEntry<'a>> {
        let mut result = Vec::new();
        let secondary_map: HashMap<String, &LogEntry> = self
            .secondary_entries
            .iter()
            .filter_map(|entry| {
                entry.metadata()
                    .as_ref()
                    .and_then(|m| m.get(key))
                    .map(|v| (v.to_string(), entry))
            })
            .collect();

        for primary_entry in self.primary_entries {
            let matching_secondary = primary_entry
                .metadata()
                .as_ref()
                .and_then(|m| m.get(key))
                .and_then(|v| secondary_map.get(&v.to_string()))
                .copied();

            result.push(CombinedEntry {
                primary: primary_entry,
                secondary: matching_secondary,
            });
        }

        result
    }

    /// Merge entries from both sources into a single timeline
    pub fn merge_chronologically(&self) -> Vec<&'a LogEntry> {
        let mut all_entries: Vec<&'a LogEntry> = Vec::new();
        all_entries.extend(self.primary_entries);
        all_entries.extend(self.secondary_entries);
        
        all_entries.sort_by_key(|entry| *entry.timestamp());
        all_entries
    }

    /// Combine entries based on matching source field
    pub fn combine_by_source(&self) -> HashMap<String, Vec<&'a LogEntry>> {
        let mut source_groups: HashMap<String, Vec<&'a LogEntry>> = HashMap::new();

        // Process primary entries
        for entry in self.primary_entries {
            if let Some(source) = entry.source() {
                source_groups.entry(source.clone()).or_default().push(entry);
            }
        }

        // Process secondary entries
        for entry in self.secondary_entries {
            if let Some(source) = entry.source() {
                source_groups.entry(source.clone()).or_default().push(entry);
            }
        }

        source_groups
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
        metadata: Option<serde_json::Value>,
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
    fn test_combine_by_timestamp() {
        let primary_entries = vec![
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
                LogLevel::Info,
                "action1",
                None,
                None,
            ),
        ];

        let secondary_entries = vec![
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 1).unwrap(),
                LogLevel::Info,
                "action2",
                None,
                None,
            ),
        ];

        let combiner = LogCombiner::new(&primary_entries, &secondary_entries);
        let combined = combiner.combine_by_timestamp(Duration::seconds(2));

        assert_eq!(combined.len(), 1);
        assert!(combined[0].secondary.is_some());
    }

    #[test]
    fn test_combine_by_metadata_key() {
        let primary_entries = vec![
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
                LogLevel::Info,
                "action1",
                None,
                Some(json!({"request_id": "123"})),
            ),
        ];

        let secondary_entries = vec![
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 1).unwrap(),
                LogLevel::Info,
                "action2",
                None,
                Some(json!({"request_id": "123"})),
            ),
        ];

        let combiner = LogCombiner::new(&primary_entries, &secondary_entries);
        let combined = combiner.combine_by_metadata_key("request_id");

        assert_eq!(combined.len(), 1);
        assert!(combined[0].secondary.is_some());
    }

    #[test]
    fn test_merge_chronologically() {
        let primary_entries = vec![
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
                LogLevel::Info,
                "action1",
                None,
                None,
            ),
        ];

        let secondary_entries = vec![
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 1).unwrap(),
                LogLevel::Info,
                "action2",
                None,
                None,
            ),
        ];

        let combiner = LogCombiner::new(&primary_entries, &secondary_entries);
        let merged = combiner.merge_chronologically();

        assert_eq!(merged.len(), 2);
        assert!(merged[0].timestamp() <= merged[1].timestamp());
    }

    #[test]
    fn test_combine_by_source() {
        let primary_entries = vec![
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
                LogLevel::Info,
                "action1",
                Some("web"),
                None,
            ),
        ];

        let secondary_entries = vec![
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 1).unwrap(),
                LogLevel::Info,
                "action2",
                Some("web"),
                None,
            ),
        ];

        let combiner = LogCombiner::new(&primary_entries, &secondary_entries);
        let source_groups = combiner.combine_by_source();

        assert_eq!(source_groups.len(), 1);
        assert_eq!(source_groups.get("web").unwrap().len(), 2);
    }
}