use std::path::PathBuf;
use tracing;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    filter::EnvFilter,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer,
};

/// The log level configuration for the application.
///
/// This enum represents different log level configurations that can be used
/// in the application. Each variant corresponds to a specific environment
/// or use case.
#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    /// Development environment with detailed logging
    Development,
    /// Production environment with minimal logging
    Production,
    /// Testing environment with specific logging requirements
    Testing,
}

impl LogLevel {
    /// Returns the string representation of the log level.
    ///
    /// # Returns
    /// * `&'static str` - The string representation of the log level
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Development => "debug",
            LogLevel::Production => "info",
            LogLevel::Testing => "warn",
        }
    }
}

/// Initializes the logging system for the application.
///
/// This function sets up the logging system with the following components:
/// 1. A console logger for development
/// 2. A file logger for production
/// 3. Environment variable based filtering
///
/// # Arguments
/// * `log_dir` - Optional directory path for log files
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Ok(()) if successful, or an error if initialization fails
pub fn init_logging(log_dir: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    // Create console layer
    let console_layer = fmt::layer()
        .with_target(false)
        .with_level(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_filter(EnvFilter::from_default_env());

    // Create file layer if log directory is provided
    let file_layer = if let Some(dir) = log_dir {
        let file_appender = RollingFileAppender::new(Rotation::DAILY, dir, "dds.log");
        let file_layer = fmt::layer()
            .with_target(false)
            .with_level(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .with_span_events(FmtSpan::CLOSE)
            .with_writer(file_appender)
            .with_filter(EnvFilter::from_default_env());
        Some(file_layer)
    } else {
        None
    };

    // Initialize the subscriber with both layers
    let subscriber = tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer);

    subscriber.init();

    Ok(())
}
