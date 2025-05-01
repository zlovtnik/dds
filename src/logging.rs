use std::path::PathBuf;
use tracing::{Level, Subscriber};
use tracing_subscriber::{
    filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer,
};

/// Initialize the logging system with both console and file output
pub fn init_logging(log_dir: Option<PathBuf>) -> anyhow::Result<()> {
    // Create a layer for console output
    let console_layer = fmt::layer()
        .with_target(false)
        .with_level(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_filter(EnvFilter::from_default_env());

    // Create a subscriber with the console layer
    let subscriber = tracing_subscriber::registry().with(console_layer);

    // If a log directory is provided, add file logging
    if let Some(log_dir) = log_dir {
        // Ensure the log directory exists
        std::fs::create_dir_all(&log_dir)?;

        // Create a rolling file appender
        let file_appender = tracing_appender::rolling::RollingFileAppender::new(
            tracing_appender::rolling::RollingFileAppenderBuilder::new("app", &log_dir)
                .rotation(tracing_appender::rolling::Rotation::DAILY)
                .max_log_files(7),
        );

        // Create a layer for file output
        let file_layer = fmt::layer()
            .with_target(false)
            .with_level(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .with_writer(file_appender)
            .with_filter(EnvFilter::from_default_env());

        // Add the file layer to the subscriber
        subscriber.with(file_layer).init();
    } else {
        // Initialize with just console logging
        subscriber.init();
    }

    Ok(())
}

/// Log levels for different environments
pub enum LogLevel {
    Development,
    Production,
    Testing,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Development => "debug",
            LogLevel::Production => "info",
            LogLevel::Testing => "warn",
        }
    }
}
