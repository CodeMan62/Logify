use clap::Parser;
use logify::{
    cli::{Cli, Commands},
    error::LogifyError,
    export::{ExportFormat, LogExporter},
    filter::LogFilter,
    parser::LogParser,
};

fn main() -> Result<(), LogifyError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Process {
            input,
            output,
            format,
            level,
            source,
            start_time,
            end_time,
        } => {
            // Parse input file
            let parser = LogParser::new();
            let entries = parser.parse_file(&input)?;

            // Apply filters
            let mut filter = LogFilter::new(entries);

            if let Some(level_str) = level {
                let log_level = level_str.parse()?;
                filter = filter.by_level(&log_level);
            }

            if let (Some(start), Some(end)) = (start_time, end_time) {
                let start = start.parse()?;
                let end = end.parse()?;
                filter = filter.by_time_range(start, end);
            }

            if let Some(src) = source {
                filter = filter.by_source(&src);
            }

            let filtered_entries = filter.entries();
            let exporter = LogExporter::new(filtered_entries);

            // Export results
            let format = format.parse::<ExportFormat>()?;

            match output {
                Some(path) => {
                    exporter.export_to_file(&path, format)?;
                }
                None => {
                    let output = exporter.export_to_string(format)?;
                    println!("{}", output);
                }
            }
        }
    }

    Ok(())
}
