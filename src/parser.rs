
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub user_id: u32,
    pub action: String,
    pub details: Value
}

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("CSV parsing error: {0}")]
    CsvParse(String),
    
    #[error("JSON parsing error: {0}")]
    JsonParse(#[from] serde_json::Error),
    
    #[error("DateTime parsing error: {0}")]
    DateTimeParse(#[from] chrono::ParseError),
    
    #[error("Invalid file extension")]
    InvalidFileExtension,
}

pub struct LogParser;

impl LogParser {
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Vec<LogEntry>, ParserError> {
        let extension = path.as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or(ParserError::InvalidFileExtension)?;

        match extension.to_lowercase().as_str() {
            "csv" => Self::parse_csv(path),
            "json" => Self::parse_json(path),
            _ => Err(ParserError::InvalidFileExtension)
        }
    }

    fn parse_csv<P: AsRef<Path>>(path: P) -> Result<Vec<LogEntry>, ParserError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();
        let mut lines = reader.lines();
        
        // Skip header line
        let _ = lines.next();

        for (line_num, line) in lines.enumerate() {
            let line = line?;
            let fields: Vec<&str> = line.split(',').collect();
            
            if fields.len() != 4 {
                return Err(ParserError::CsvParse(
                    format!("Invalid number of fields at line {}", line_num + 2)
                ));
            }

            let entry = Self::parse_csv_line(&fields)
                .map_err(|e| ParserError::CsvParse(
                    format!("Error at line {}: {}", line_num + 2, e)
                ))?;
                
            entries.push(entry);
        }

        Ok(entries)
    }

    fn parse_json<P: AsRef<Path>>(path: P) -> Result<Vec<LogEntry>, ParserError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let entries: Vec<LogEntry> = serde_json::from_reader(reader)?;
        Ok(entries)
    }

    fn parse_csv_line(fields: &[&str]) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let timestamp = fields[0].parse::<DateTime<Utc>>()?;
        let user_id = fields[1].parse::<u32>()?;
        let action = fields[2].to_string();
        let details: Value = serde_json::from_str(fields[3])?;

        Ok(LogEntry {
            timestamp,
            user_id,
            action,
            details
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_parse_csv() {
        let csv_content = r#"timestamp,user_id,action,details
2024-11-11T13:45:30Z,1001,login,{"device":"mobile","location":"New York"}"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_content).unwrap();
        
        let entries = LogParser::parse_file(temp_file.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].user_id, 1001);
        assert_eq!(entries[0].action, "login");
    }

    #[test]
    fn test_parse_json() {
        let json_content = r#"[
            {
                "timestamp": "2024-11-11T13:45:30Z",
                "user_id": 1001,
                "action": "login",
                "details": {"device":"mobile","location":"New York"}
            }
        ]"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.as_file_mut().write_all(json_content.as_bytes()).unwrap();
        temp_file.as_file_mut().write_all(b"\n").unwrap();
        
        let entries = LogParser::parse_file(temp_file.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].user_id, 1001);
        assert_eq!(entries[0].action, "login");
    }
}

