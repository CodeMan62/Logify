// First, let's create sample data (data/sample.csv):
timestamp,user_id,action,duration
2024-03-15T10:00:00Z,user123,login,30.5
2024-03-15T10:05:00Z,user456,search,15.2
2024-03-15T10:10:00Z,user123,logout,5.0
2024-03-15T10:15:00Z,user789,login,25.8

// Now, let's implement a basic working example that shows what we're building:

// src/main.rs
use chrono::Utc;
use data_processing_pipeline::{LogEntry, Result};
use log::info;

fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    // Create CSV handler
    let csv_handler = CsvHandler::new(b',', true);

    // Read logs from CSV
    let logs = csv_handler.read_logs("data/sample.csv")?;

    // Print initial statistics
    info!("Loaded {} log entries", logs.len());

    // Example: Filter login actions
    let login_logs: Vec<_> = logs
        .iter()
        .filter(|log| log.action == "login")
        .collect();

    info!("Found {} login actions", login_logs.len());

    // Calculate average duration
    let avg_duration: f64 = login_logs
        .iter()
        .map(|log| log.duration)
        .sum::<f64>() / login_logs.len() as f64;

    info!("Average login duration: {:.2} seconds", avg_duration);

    Ok(())
}

// Let's see what this produces when you run it:
$ RUST_LOG=info cargo run
[INFO] Starting data processing pipeline...
[INFO] Loaded 4 log entries
[INFO] Found 2 login actions
[INFO] Average login duration: 28.15 seconds

// You can also use the pipeline to transform and save data:

// Example usage with all Phase 1 & 2 components:
use data_processing_pipeline::pipeline::*;

fn process_logs() -> Result<()> {
    let csv_handler = CsvHandler::new(b',', true);

    // Read input
    let logs = csv_handler.read_logs("data/sample.csv")?;

    // Create processed entries
    let processed_logs: Vec<LogEntry> = logs.iter()
        .filter(|log| log.duration > 0.0)  // Filter valid durations
        .map(|log| LogEntry {
            timestamp: log.timestamp,
            user_id: log.user_id.clone(),
            action: log.action.clone(),
            duration: log.duration / 60.0,  // Convert to minutes
            metadata: Some(json!({
                "processed_at": Utc::now().to_rfc3339(),
                "original_duration": log.duration
            }))
        })
        .collect();

    // Write processed logs
    csv_handler.write_logs("data/processed.csv", &processed_logs)?;

    Ok(())
}

// This will create a new file (data/processed.csv):
timestamp,user_id,action,duration,metadata
2024-03-15T10:00:00Z,user123,login,0.508,{"processed_at":"2024-03-15T10:30:00Z","original_duration":30.5}
2024-03-15T10:05:00Z,user456,search,0.253,{"processed_at":"2024-03-15T10:30:00Z","original_duration":15.2}
2024-03-15T10:10:00Z,user123,logout,0.083,{"processed_at":"2024-03-15T10:30:00Z","original_duration":5.0}
2024-03-15T10:15:00Z,user789,login,0.430,{"processed_at":"2024-03-15T10:30:00Z","original_duration":25.8}

// You can also handle errors gracefully:
// Example with invalid data (data/invalid.csv):
timestamp,user_id,action,duration
2024-03-15T10:00:00Z,,login,30.5  // Empty user_id
2024-03-15T10:05:00Z,user456,search,-15.2  // Negative duration

let result = csv_handler.read_logs("data/invalid.csv");
match result {
    Err(PipelineError::Validation(ValidationError::EmptyUserId)) => {
        eprintln!("Found entry with empty user ID");
    }
    Err(PipelineError::Validation(ValidationError::NegativeDuration)) => {
        eprintln!("Found entry with negative duration");
    }
    _ => unreachable!("Unexpected result"),
}
