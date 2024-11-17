use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Duration, Utc};
use serde_json::Value;

use crate::parser::{LogEntry, LogLevel};

pub struct LogAnalyzer<'a> {
    entries: &'a [LogEntry],
}

#[derive(Debug)]
pub struct TimeSeriesData {
    pub timestamp: DateTime<Utc>,
    pub count: usize,
    pub level_distribution: HashMap<LogLevel, usize>,
}

#[derive(Debug)]
pub struct PatternAnalysis {
    pub pattern: String,
    pub occurrences: usize,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub related_actions: HashSet<String>,
}

#[derive(Debug)]
pub struct ErrorAnalysis {
    pub error_code: String,
    pub frequency: usize,
    pub first_occurrence: DateTime<Utc>,
    pub last_occurrence: DateTime<Utc>,
    pub related_messages: Vec<String>,
}

impl<'a> LogAnalyzer<'a> {
    pub fn new(entries: &'a [LogEntry]) -> Self {
        LogAnalyzer { entries }
    }

    /// Generate time series data with custom time windows
    pub fn generate_time_series(&self, window: Duration) -> Vec<TimeSeriesData> {
        if self.entries.is_empty() {
            return Vec::new();
        }

        let mut series = Vec::new();
        let mut current_entries = Vec::new();
        let mut window_start = *self.entries[0].timestamp();
        let mut window_end = window_start + window;

        for entry in self.entries {
            while entry.timestamp() > &window_end {
                if !current_entries.is_empty() {
                    series.push(self.create_time_series_data(
                        window_start,
                        &current_entries,
                    ));
                }
                window_start = window_end;
                window_end = window_start + window;
                current_entries.clear();
            }
            current_entries.push(entry);
        }

        // Handle the last window
        if !current_entries.is_empty() {
            series.push(self.create_time_series_data(window_start, &current_entries));
        }

        series
    }

    /// Detect patterns in log messages
    pub fn detect_patterns(&self, min_occurrences: usize) -> Vec<PatternAnalysis> {
        let mut patterns: HashMap<String, Vec<&LogEntry>> = HashMap::new();

        // Group similar messages
        for entry in self.entries {
            let pattern = self.extract_message_pattern(entry.message());
            patterns.entry(pattern).or_default().push(entry);
        }

        // Create pattern analysis for frequent patterns
        patterns
            .into_iter()
            .filter(|(_, entries)| entries.len() >= min_occurrences)
            .map(|(pattern, entries)| {
                let first_seen = entries.iter().map(|e| e.timestamp()).min().copied().unwrap();
                let last_seen = entries.iter().map(|e| e.timestamp()).max().copied().unwrap();
                let related_actions: HashSet<String> = entries
                    .iter()
                    .map(|e| e.action().to_string())
                    .collect();

                PatternAnalysis {
                    pattern,
                    occurrences: entries.len(),
                    first_seen,
                    last_seen,
                    related_actions,
                }
            })
            .collect()
    }

    /// Analyze error patterns and frequencies
    pub fn analyze_errors(&self) -> Vec<ErrorAnalysis> {
        let mut error_groups: HashMap<String, Vec<&LogEntry>> = HashMap::new();

        // Group errors by error code
        for entry in self.entries {
            if entry.level() == &LogLevel::Error {
                let error_code = self.extract_error_code(entry);
                error_groups.entry(error_code).or_default().push(entry);
            }
        }

        // Create error analysis for each group
        error_groups
            .into_iter()
            .map(|(error_code, entries)| {
                ErrorAnalysis {
                    error_code,
                    frequency: entries.len(),
                    first_occurrence: *entries.iter().map(|e| e.timestamp()).min().unwrap(),
                    last_occurrence: *entries.iter().map(|e| e.timestamp()).max().unwrap(),
                    related_messages: entries.iter().map(|e| e.message().to_string()).collect(),
                }
            })
            .collect()
    }

    /// Detect anomalies in log frequency
    pub fn detect_anomalies(&self, window: Duration, threshold: f64) -> Vec<DateTime<Utc>> {
        let time_series = self.generate_time_series(window);
        if time_series.len() < 2 {
            return Vec::new();
        }

        // Calculate mean and standard deviation of counts
        let counts: Vec<f64> = time_series.iter().map(|ts| ts.count as f64).collect();
        let mean = counts.iter().sum::<f64>() / counts.len() as f64;
        let variance = counts.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / counts.len() as f64;
        let std_dev = variance.sqrt();

        // Detect anomalies
        time_series
            .iter()
            .filter(|ts| {
                let z_score = (ts.count as f64 - mean).abs() / std_dev;
                z_score > threshold
            })
            .map(|ts| ts.timestamp)
            .collect()
    }

    // Helper methods
    fn create_time_series_data(&self, timestamp: DateTime<Utc>, entries: &[&LogEntry]) -> TimeSeriesData {
        let mut level_distribution = HashMap::new();
        for entry in entries {
            *level_distribution.entry(entry.level().clone()).or_insert(0) += 1;
        }

        TimeSeriesData {
            timestamp,
            count: entries.len(),
            level_distribution,
        }
    }

    fn extract_message_pattern(&self, message: &str) -> String {
        // Simple pattern extraction: replace numbers with #
        message
            .split_whitespace()
            .map(|word| {
                if word.chars().all(|c| c.is_numeric()) {
                    "#".to_string()
                } else {
                    word.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn extract_error_code(&self, entry: &LogEntry) -> String {
        entry
            .metadata()
            .as_ref()
            .and_then(|m| m.get("error_code"))
            .map(|v| v.to_string())
            .unwrap_or_else(|| "UNKNOWN".to_string())
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
        message: &str,
        action: &str,
        metadata: Option<Value>,
    ) -> LogEntry {
        LogEntry::new(
            timestamp,
            level,
            message.to_string(),
            action.to_string(),
            Some("test_source".to_string()),
            metadata,
        )
    }

    #[test]
    fn test_time_series_generation() {
        let entries = vec![
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
                LogLevel::Info,
                "Test message 1",
                "action1",
                None,
            ),
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 30).unwrap(),
                LogLevel::Error,
                "Test message 2",
                "action2",
                None,
            ),
        ];

        let analyzer = LogAnalyzer::new(&entries);
        let series = analyzer.generate_time_series(Duration::hours(1));
        
        assert_eq!(series.len(), 1);
        assert_eq!(series[0].count, 2);
    }

    #[test]
    fn test_pattern_detection() {
        let entries = vec![
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
                LogLevel::Info,
                "User 123 logged in",
                "login",
                None,
            ),
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 1).unwrap(),
                LogLevel::Info,
                "User 456 logged in",
                "login",
                None,
            ),
        ];

        let analyzer = LogAnalyzer::new(&entries);
        let patterns = analyzer.detect_patterns(2);
        
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].occurrences, 2);
    }

    #[test]
    fn test_error_analysis() {
        let entries = vec![
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
                LogLevel::Error,
                "Database connection failed",
                "db_connect",
                Some(json!({"error_code": "DB001"})),
            ),
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 1).unwrap(),
                LogLevel::Error,
                "Database connection failed again",
                "db_connect",
                Some(json!({"error_code": "DB001"})),
            ),
        ];

        let analyzer = LogAnalyzer::new(&entries);
        let error_analysis = analyzer.analyze_errors();
        
        assert_eq!(error_analysis.len(), 1);
        assert_eq!(error_analysis[0].frequency, 2);
    }
}