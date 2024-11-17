pub mod analyze;
pub mod cli;
pub mod combine;
pub mod config;
pub mod error;
pub mod export;
pub mod filter;
pub mod parser;
pub mod transformers;

pub use analyze::LogAnalyzer;
pub use cli::{Cli, Commands};
pub use combine::LogCombiner;
pub use config::LogifyConfig;
pub use error::{LogifyError, Result};
pub use export::LogExporter;
pub use filter::LogFilter;
pub use parser::LogEntry;
pub use transformers::LogTransformer;

/// Version of the Logify library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the Logify library with default configuration
pub fn init() -> Result<LogifyConfig> {
    LogifyConfig::load()
}

/// Initialize the Logify library with a custom configuration file
pub fn init_with_config<P: AsRef<std::path::Path>>(config_path: P) -> Result<LogifyConfig> {
    LogifyConfig::from_file(config_path)
}