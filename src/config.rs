use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use config::{Config, ConfigError, Environment, File};

use crate::error::{LogifyError, Result};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogifyConfig {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub filter: FilterConfig,
    #[serde(default)]
    pub export: ExportConfig,
    #[serde(default)]
    pub analysis: AnalysisConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneralConfig {
    pub default_log_path: Option<String>,
    pub max_file_size: Option<usize>,
    pub verbose: bool,
    pub timezone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilterConfig {
    pub default_level: Option<String>,
    pub exclude_patterns: Vec<String>,
    pub include_patterns: Vec<String>,
    pub max_age_days: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExportConfig {
    pub default_format: String,
    pub output_directory: Option<String>,
    pub max_batch_size: usize,
    pub compress: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnalysisConfig {
    pub time_window_minutes: u32,
    pub min_pattern_occurrences: usize,
    pub anomaly_threshold: f64,
    pub max_patterns: usize,
}

impl Default for LogifyConfig {
    fn default() -> Self {
        LogifyConfig {
            general: GeneralConfig::default(),
            filter: FilterConfig::default(),
            export: ExportConfig::default(),
            analysis: AnalysisConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        GeneralConfig {
            default_log_path: None,
            max_file_size: Some(100 * 1024 * 1024), // 100MB
            verbose: false,
            timezone: Some("UTC".to_string()),
        }
    }
}

impl Default for FilterConfig {
    fn default() -> Self {
        FilterConfig {
            default_level: Some("info".to_string()),
            exclude_patterns: Vec::new(),
            include_patterns: Vec::new(),
            max_age_days: Some(30),
        }
    }
}

impl Default for ExportConfig {
    fn default() -> Self {
        ExportConfig {
            default_format: "json".to_string(),
            output_directory: None,
            max_batch_size: 1000,
            compress: false,
        }
    }
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        AnalysisConfig {
            time_window_minutes: 60,
            min_pattern_occurrences: 5,
            anomaly_threshold: 2.0,
            max_patterns: 100,
        }
    }
}

impl LogifyConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config_str = fs::read_to_string(path)
            .map_err(|e| LogifyError::ConfigError(format!("Failed to read config file: {}", e)))?;

        serde_json::from_str(&config_str)
            .map_err(|e| LogifyError::ConfigError(format!("Failed to parse config file: {}", e)))
    }

    pub fn load() -> Result<Self> {
        let mut builder = Config::builder()
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name("config/local").required(false))
            .add_source(Environment::with_prefix("LOGIFY"));

        let config = builder
            .build()
            .map_err(|e| LogifyError::ConfigError(format!("Failed to build config: {}", e)))?;

        config
            .try_deserialize()
            .map_err(|e| LogifyError::ConfigError(format!("Failed to deserialize config: {}", e)))
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let config_str = serde_json::to_string_pretty(self)
            .map_err(|e| LogifyError::ConfigError(format!("Failed to serialize config: {}", e)))?;

        fs::write(path, config_str)
            .map_err(|e| LogifyError::ConfigError(format!("Failed to write config file: {}", e)))
    }

    pub fn merge(&mut self, other: LogifyConfig) {
        if let Some(path) = other.general.default_log_path {
            self.general.default_log_path = Some(path);
        }
        if let Some(size) = other.general.max_file_size {
            self.general.max_file_size = Some(size);
        }
        self.general.verbose = other.general.verbose;
        if let Some(tz) = other.general.timezone {
            self.general.timezone = Some(tz);
        }

        // Merge filter config
        if let Some(level) = other.filter.default_level {
            self.filter.default_level = Some(level);
        }
        self.filter.exclude_patterns.extend(other.filter.exclude_patterns);
        self.filter.include_patterns.extend(other.filter.include_patterns);
        if let Some(age) = other.filter.max_age_days {
            self.filter.max_age_days = Some(age);
        }

        // Merge export config
        self.export.default_format = other.export.default_format;
        if let Some(dir) = other.export.output_directory {
            self.export.output_directory = Some(dir);
        }
        self.export.max_batch_size = other.export.max_batch_size;
        self.export.compress = other.export.compress;

        // Merge analysis config
        self.analysis = other.analysis;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = LogifyConfig::default();
        assert_eq!(config.export.default_format, "json");
        assert_eq!(config.analysis.time_window_minutes, 60);
    }

    #[test]
    fn test_save_and_load_config() {
        let config = LogifyConfig::default();
        let temp_file = NamedTempFile::new().unwrap();
        
        config.save(temp_file.path()).unwrap();
        let loaded_config = LogifyConfig::from_file(temp_file.path()).unwrap();
        
        assert_eq!(
            config.export.default_format,
            loaded_config.export.default_format
        );
    }

    #[test]
    fn test_merge_configs() {
        let mut base_config = LogifyConfig::default();
        let mut other_config = LogifyConfig::default();
        
        other_config.general.verbose = true;
        other_config.export.default_format = "csv".to_string();
        
        base_config.merge(other_config);
        
        assert!(base_config.general.verbose);
        assert_eq!(base_config.export.default_format, "csv");
    }
}