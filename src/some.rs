use chrono::{DateTime, Utc};
use serde_json::Value;
struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub user_id: u32,
    pub action: String,
    pub details: Value 
}