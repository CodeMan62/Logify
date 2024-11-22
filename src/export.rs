use crate::error::LogifyError;
use crate::parser::LogEntry;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    Csv,
    Text,
}

impl FromStr for ExportFormat {
    type Err = LogifyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(ExportFormat::Json),
            "csv" => Ok(ExportFormat::Csv),
            "text" => Ok(ExportFormat::Text),
            _ => Err(LogifyError::InvalidFormat(s.to_string())),
        }
    }
}

pub struct LogExporter {
    entries: Vec<LogEntry>,
}

impl LogExporter {
    pub fn new(entries: Vec<LogEntry>) -> Self {
        Self { entries }
    }

    pub fn export_to_file(&self, path: &Path, format: ExportFormat) -> Result<(), LogifyError> {
        let content = self.export_to_string(format)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn export_to_string(&self, format: ExportFormat) -> Result<String, LogifyError> {
        match format {
            ExportFormat::Json => {
                serde_json::to_string_pretty(&self.entries).map_err(LogifyError::Json)
            }
            ExportFormat::Csv => {
                let mut wtr = csv::Writer::from_writer(vec![]);
                for entry in &self.entries {
                    wtr.serialize(entry).map_err(LogifyError::Csv)?;
                }
                String::from_utf8(wtr.into_inner().map_err(LogifyError::Csv)?)
                    .map_err(|e| LogifyError::Parser(e.to_string()))
            }
            ExportFormat::Text => {
                let output = self.entries
                    .iter()
                    .map(|entry| format!("[{}] {} - {}", entry.level, entry.timestamp, entry.message))
                    .collect::<Vec<_>>()
                    .join("\n");
                Ok(output)
            }
        }
    }
}
