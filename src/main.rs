mod parser;
mod filter;
mod utils;
mod processer;
mod errors;
mod transformers;
mod some;
use crate::parser::{LogParser, LogEntry};

fn main() {
    // Parse a CSV file
    match LogParser::parse_file("data/log.csv") {
        Ok(entries) => {
            println!("Successfully parsed {} log entries", entries.len());
            
            // Process each log entry
            for entry in entries {
                println!("Time: {}", entry.timestamp);
                println!("User: {}", entry.user_id);
                println!("Action: {}", entry.action);
                println!("---");
            }
        }
        Err(e) => eprintln!("Error parsing file: {}", e),
    }
}