use chrono::{TimeZone, Utc};
use logify::{
    LogAnalyzer, LogCombiner, LogEntry, LogExporter, LogFilter, LogTransformer, LogifyConfig,
    LogifyError,
};
use serde_json::json;
use std::path::PathBuf;
use tempfile::TempDir;

// Helper function to create test log entries
fn create_test_logs() -> Vec<LogEntry> {
    vec![
        LogEntry::new(
            Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
            "INFO".parse().unwrap(),
            "User login successful".to_string(),
            "login".to_string(),
            Some("auth".to_string()),
            Some(json!({"user_id": "123"})),
        ),
        LogEntry::new(
            Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 1).unwrap(),
            "ERROR".parse().unwrap(),
            "Database connection failed".to_string(),
            "db_connect".to_string(),
            Some("database".to_string()),
            Some(json!({"error_code": "DB001"})),
        ),
    ]
}

#[test]
fn test_end_to_end_analysis() -> Result<(), LogifyError> {
    let entries = create_test_logs();
    let temp_dir = TempDir::new()?;
    let output_path = temp_dir.path().join("analysis.json");

    // Analyze
    let analyzer = LogAnalyzer::new(&entries);
    let patterns = analyzer.detect_patterns(1);
    assert!(!patterns.is_empty());

    // Export results
    let exporter = LogExporter::new(&entries);
    exporter.export_to_file(&output_path, "json".into())?;

    assert!(output_path.exists());
    Ok(())
}

#[test]
fn test_end_to_end_filtering() -> Result<(), LogifyError> {
    let entries = create_test_logs();
    
    // Filter
    let filter = LogFilter::new(&entries);
    let error_logs = filter.by_level("ERROR".parse()?);
    
    assert_eq!(error_logs.len(), 1);
    assert_eq!(error_logs[0].message(), "Database connection failed");
    
    Ok(())
}

#[test]
fn test_end_to_end_transformation() -> Result<(), LogifyError> {
    let entries = create_test_logs();
    
    // Transform
    let transformer = LogTransformer::new(&entries);
    let transformed = transformer.map(|entry| entry.message().to_string());
    
    assert_eq!(transformed.len(), 2);
    assert!(transformed.contains(&"User login successful".to_string()));
    
    Ok(())
}

#[test]
fn test_config_loading() -> Result<(), LogifyError> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("config.json");
    
    // Create and save config
    let config = LogifyConfig::default();
    config.save(&config_path)?;
    
    // Load config
    let loaded_config = LogifyConfig::from_file(&config_path)?;
    assert_eq!(
        loaded_config.export.default_format,
        config.export.default_format
    );
    
    Ok(())
}

#[test]
fn test_error_handling() {
    let result = LogifyConfig::from_file(PathBuf::from("nonexistent.json"));
    assert!(result.is_err());
}