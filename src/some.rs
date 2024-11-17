use core::error;
use std::io;

use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::errors;
struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub user_id: u32,
    pub action: String,
    pub details: Value 
}

pub enum ParseError {
    #[error()]
    Io(#[from] io::Error)
}