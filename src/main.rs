use clap::Parser;
use logify::{
    cli::{Cli, Commands},
    config::LogifyConfig,
    error::{LogifyError, Result},
    export::{ExportFormat, ExportError},
    parser::{LogLevel, LogLevelParseError},
    LogAnalyzer, LogCombiner, LogExporter, LogFilter, LogTransformer,
};
use std::path::PathBuf;
use std::process;
use std::str::FromStr;
use chrono::{DateTime, Utc};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let config = load_config(cli.config)?;

    match cli.command {
        Commands::Analyze {
            input,
            window,
            min_occurrences,
            format,
            output,
        } => handle_analyze(
            input,
            window,
            min_occurrences,
            &format,
            output,
            cli.verbose,
            &config,
        ),
        Commands::Filter {
            input,
            level,
            time_range,
            source,
            output,
        } => handle_filter(input, level, time_range, source, output, cli.verbose, &config),
        Commands::Export {
            input,
            format,
            output,
        } => handle_export(input, &format, output, cli.verbose, &config),
    }
}

fn load_config(config_path: Option<PathBuf>) -> Result<LogifyConfig> {
    match config_path {
        Some(path) => LogifyConfig::from_file(&path),
        None => LogifyConfig::load(),
    }
}

fn handle_analyze(
    input: PathBuf,
    window: u64,
    min_occurrences: usize,
    format: &str,
    output: Option<PathBuf>,
    verbose: bool,
    config: &LogifyConfig,
) -> Result<()> {
    if verbose {
        println!("Analyzing log file: {:?}", input);
    }

    let entries = logify::parser::parse_log_file(&input)?;
    let analyzer = LogAnalyzer::new(&entries);

    // Perform analysis
    let time_series = analyzer.generate_time_series(chrono::Duration::minutes(window as i64));
    let patterns = analyzer.detect_patterns(min_occurrences);
    let error_analysis = analyzer.analyze_errors();

    // Export results
    let exporter = LogExporter::new(&entries);
    let format = ExportFormat::from_str(format)
        .map_err(|e| LogifyError::FormatError(e.to_string()))?;

    match output {
        Some(path) => {
            exporter.export_to_file(&path, format)?;
            if verbose {
                println!("Analysis results written to: {:?}", path);
            }
        }
        None => {
            let output = exporter.export_to_string(format)?;
            println!("{}", output);
        }
    }

    Ok(())
}

fn parse_time_range(range: &str) -> Result<(DateTime<Utc>, DateTime<Utc>)> {
    let parts: Vec<&str> = range.split('/').collect();
    if parts.len() != 2 {
        return Err(LogifyError::ValidationError(
            "Time range must be in format 'start/end'".to_string(),
        ));
    }

    let start = DateTime::parse_from_rfc3339(parts[0])
        .map_err(|e| LogifyError::ValidationError(format!("Invalid start time: {}", e)))?
        .with_timezone(&Utc);
    let end = DateTime::parse_from_rfc3339(parts[1])
        .map_err(|e| LogifyError::ValidationError(format!("Invalid end time: {}", e)))?
        .with_timezone(&Utc);

    Ok((start, end))
}

fn handle_filter(
    input: PathBuf,
    level: Option<String>,
    time_range: Option<String>,
    source: Option<String>,
    output: Option<PathBuf>,
    verbose: bool,
    config: &LogifyConfig,
) -> Result<()> {
    if verbose {
        println!("Filtering log file: {:?}", input);
    }

    let entries = logify::parser::parse_log_file(&input)?;
    let mut filter = LogFilter::new(&entries);

    // Apply filters
    if let Some(level_str) = level {
        let log_level = LogLevel::from_str(&level_str)
            .map_err(|_| LogifyError::ValidationError(format!("Invalid log level: {}", level_str)))?;
        filter = filter.filter_by_level(&log_level);
    }

    if let Some(range) = time_range {
        let (start, end) = parse_time_range(&range)?;
        filter = filter.filter_by_time_range(start, end);
    }

    if let Some(src) = source {
        filter = filter.filter_by_source(&src);
    }

    let filtered_entries = filter.get_entries();

    // Export filtered results
    let exporter = LogExporter::new(filtered_entries);
    let format = ExportFormat::from_str(&config.export.default_format)
        .map_err(|e| LogifyError::FormatError(e.to_string()))?;

    match output {
        Some(path) => {
            exporter.export_to_file(&path, format)?;
            if verbose {
                println!("Filtered results written to: {:?}", path);
            }
        }
        None => {
            let output = exporter.export_to_string(format)?;
            println!("{}", output);
        }
    }

    Ok(())
}

fn handle_export(
    input: PathBuf,
    format: &str,
    output: PathBuf,
    verbose: bool,
    _config: &LogifyConfig,
) -> Result<()> {
    if verbose {
        println!("Exporting log file: {:?}", input);
    }

    let entries = logify::parser::parse_log_file(&input)?;
    let exporter = LogExporter::new(&entries);
    let format = ExportFormat::from_str(format)
        .map_err(|e| LogifyError::FormatError(e.to_string()))?;
    
    exporter.export_to_file(&output, format)?;

    if verbose {
        println!("Logs exported to: {:?}", output);
    }

    Ok(())
}

