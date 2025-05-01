/// The main module of the application.
///
/// This module contains the entry point of the application and demonstrates the usage of
/// the database operations and ETL pipeline functionality.
mod db;
mod etl;
mod graphql;
mod logging;
mod models;

use axum::serve;
use db::DbConnection;
use dotenv::dotenv;
use etl::ETLPipeline;
use graphql::{create_router, create_schema};
use logging::{init_logging, LogLevel};
use models::user::{CreateUser, UpdateUser};
use std::path::{Path, PathBuf};
use tokio::sync::broadcast;

/// The main entry point of the application.
///
/// This function:
/// 1. Loads environment variables from a .env file
/// 2. Initializes a database connection
/// 3. Sets up the GraphQL server
/// 4. Demonstrates CRUD operations on users
/// 5. Runs an ETL pipeline to process JSON files
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Ok(()) if successful, or an error if any operation fails
///
/// # Examples
/// ```rust
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // ... application code ...
/// }
/// ```
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv().ok();

    // Initialize logging
    let log_dir = std::env::var("LOG_DIR").ok().map(PathBuf::from);
    init_logging(log_dir)?;

    // Set log level based on environment
    let log_level = if std::env::var("RUST_LOG").is_err() {
        if cfg!(debug_assertions) {
            LogLevel::Development
        } else {
            LogLevel::Production
        }
    } else {
        LogLevel::Production
    };
    std::env::set_var("RUST_LOG", log_level.as_str());

    tracing::info!("Starting application initialization");

    // Initialize database connection
    let db = DbConnection::new().await?;
    tracing::info!("Database connection established");

    // Create event channel for GraphQL subscriptions
    let (event_sender, _) = broadcast::channel(100);
    tracing::debug!("GraphQL event channel created");

    // Create GraphQL schema and router
    let schema = create_schema(db.pool.clone(), event_sender);
    let router = create_router(schema);
    tracing::info!("GraphQL schema and router initialized");

    // Start the GraphQL server
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("Starting GraphQL server on http://localhost:{}", port);
    tracing::info!(
        "GraphiQL playground available at http://localhost:{}/graphiql",
        port
    );
    tracing::info!("Press Ctrl+C to stop the server");

    serve(listener, router).await?;

    Ok(())
}
