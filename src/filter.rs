use crate::parser::{LogEntry, LogLevel};
use chrono::{DateTime, Utc};

pub struct LogFilter {
    entries: Vec<LogEntry>,
}

impl LogFilter {
    pub fn new(entries: Vec<LogEntry>) -> Self {
        Self { entries }
    }

    pub fn by_level(mut self, level: &LogLevel) -> Self {
        self.entries.retain(|entry| entry.level == *level);
        self
    }

    pub fn by_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.entries.retain(|entry| {
            entry.timestamp >= start && entry.timestamp <= end
        });
        self
    }

    pub fn by_source(mut self, source: &str) -> Self {
        self.entries.retain(|entry| entry.source == source);
        self
    }

    pub fn entries(self) -> Vec<LogEntry> {
        self.entries
    }
}
