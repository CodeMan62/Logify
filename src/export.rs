use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use csv::WriterBuilder;

use crate::parser::{LogEntry, LogLevel};

#[derive(Debug, Serialize)]
pub struct ExportableLogEntry {
    timestamp: String,
    level: String,
    message: String,
    action: String,
    source: Option<String>,
    metadata: Option<Value>,
}

pub enum ExportFormat {
    Json,
    Csv,
    Custom(Box<dyn Fn(&LogEntry) -> String>),
}

pub struct LogExporter<'a> {
    entries: &'a [LogEntry],
}

#[derive(Debug)]
pub enum ExportError {
    IoError(io::Error),
    SerializationError(serde_json::Error),
    CsvError(csv::Error),
}

impl From<io::Error> for ExportError {
    fn from(error: io::Error) -> Self {
        ExportError::IoError(error)
    }
}

impl From<serde_json::Error> for ExportError {
    fn from(error: serde_json::Error) -> Self {
        ExportError::SerializationError(error)
    }
}


impl From<csv::Error> for ExportError {
    fn from(error: csv::Error) -> Self {
        ExportError::CsvError(error)
    }
}
impl From<csv::IntoInnerError<csv::Writer<Vec<u8>>>> for ExportError {
    fn from(error: csv::IntoInnerError<csv::Writer<Vec<u8>>>) -> Self {
        ExportError::CsvError(csv::Error::from(error.into_error()))
    }
}

impl<'a> LogExporter<'a> {
    pub fn new(entries: &'a [LogEntry]) -> Self {
        LogExporter { entries }
    }

    /// Export logs to a file in the specified format
    pub fn export_to_file(
        &self,
        path: &Path,
        format: ExportFormat,
    ) -> Result<(), ExportError> {
        match format {
            ExportFormat::Json => self.export_json(path),
            ExportFormat::Csv => self.export_csv(path),
            ExportFormat::Custom(formatter) => self.export_custom(path, formatter),
        }
    }

    /// Export logs to JSON format
    fn export_json(&self, path: &Path) -> Result<(), ExportError> {
        let exportable = self.prepare_exportable_entries();
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, &exportable)?;
        Ok(())
    }

    /// Export logs to CSV format
    fn export_csv(&self, path: &Path) -> Result<(), ExportError> {
        let exportable = self.prepare_exportable_entries();
        let mut writer = WriterBuilder::new()
            .has_headers(true)
            .from_path(path)?;

        for entry in exportable {
            writer.serialize(entry)?;
        }
        writer.flush()?;
        Ok(())
    }

    /// Export logs using a custom formatter
    fn export_custom<F>(&self, path: &Path, formatter: F) -> Result<(), ExportError>
    where
        F: Fn(&LogEntry) -> String,
    {
        let mut file = File::create(path)?;
        for entry in self.entries {
            writeln!(file, "{}", formatter(entry))?;
        }
        Ok(())
    }

    /// Export logs to a string in the specified format
    pub fn export_to_string(&self, format: ExportFormat) -> Result<String, ExportError> {
        match format {
            ExportFormat::Json => {
                let exportable = self.prepare_exportable_entries();
                Ok(serde_json::to_string_pretty(&exportable)?)
            }
            ExportFormat::Csv => {
                let mut writer = csv::Writer::from_writer(vec![]);
                let exportable = self.prepare_exportable_entries();
                for entry in exportable {
                    writer.serialize(entry)?;
                }
                let csv_data = String::from_utf8(writer.into_inner()?).unwrap();
                Ok(csv_data)
            }
            ExportFormat::Custom(formatter) => {
                Ok(self.entries
                    .iter()
                    .map(|entry| formatter(entry))
                    .collect::<Vec<_>>()
                    .join("\n"))
            }
        }
    }

    /// Prepare entries for export by converting them to a serializable format
    fn prepare_exportable_entries(&self) -> Vec<ExportableLogEntry> {
        self.entries
            .iter()
            .map(|entry| ExportableLogEntry {
                timestamp: entry.timestamp().to_rfc3339(),
                level: format!("{:?}", entry.level()),
                message: entry.message().to_string(),
                action: entry.action().to_string(),
                source: entry.source().clone(),
                metadata: entry.metadata().clone(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use serde_json::json;
    use tempfile::NamedTempFile;

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
    fn test_json_export() {
        let entries = vec![
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
                LogLevel::Info,
                "Test message",
                Some(json!({"key": "value"})),
            ),
        ];

        let exporter = LogExporter::new(&entries);
        let temp_file = NamedTempFile::new().unwrap();
        
        assert!(exporter.export_to_file(temp_file.path(), ExportFormat::Json).is_ok());
    }

    #[test]
    fn test_csv_export() {
        let entries = vec![
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
                LogLevel::Info,
                "Test message",
                None,
            ),
        ];

        let exporter = LogExporter::new(&entries);
        let temp_file = NamedTempFile::new().unwrap();
        
        assert!(exporter.export_to_file(temp_file.path(), ExportFormat::Csv).is_ok());
    }

    #[test]
    fn test_custom_export() {
        let entries = vec![
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
                LogLevel::Info,
                "Test message",
                None,
            ),
        ];

        let exporter = LogExporter::new(&entries);
        let temp_file = NamedTempFile::new().unwrap();
        
        let custom_formatter = |entry: &LogEntry| {
            format!("{:?} - {:?}: {:?}", 
                entry.timestamp(),
                entry.level(),
                entry.message()
            )
        };
        
        assert!(exporter
            .export_to_file(temp_file.path(), ExportFormat::Custom(Box::new(custom_formatter)))
            .is_ok());
    }

    #[test]
    fn test_export_to_string() {
        let entries = vec![
            create_test_entry(
                Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
                LogLevel::Info,
                "Test message",
                None,
            ),
        ];

        let exporter = LogExporter::new(&entries);
        let json_string = exporter.export_to_string(ExportFormat::Json);
        
        assert!(json_string.is_ok());
        assert!(json_string.unwrap().contains("Test message"));
    }
}