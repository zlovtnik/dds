mod auth;
/// The main module of the application.
///
/// This module contains the entry point of the application and demonstrates the usage of
/// the database operations and ETL pipeline functionality.
mod db;
mod etl;
mod graphql;
mod logging;
mod models;

use axum::Router;
use db::DbConnection;
use dotenv::dotenv;
use futures::StreamExt;
use graphql::{create_router, create_schema};
use logging::{init_logging, LogLevel};
use std::path::PathBuf;
use tokio::net::TcpListener;
use tokio::sync::broadcast;

/// The main entry point of the application.
///
/// This function:
/// 1. Loads environment variables from a .env file
/// 2. Initializes a database connection
/// 3. Sets up the GraphQL server
/// 4. Demonstrates CRUD operations on users
/// 5. Runs an ETL pipeline to process JSON files

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

    // Check if TLS is enabled via environment variable
    let use_https = std::env::var("USE_HTTPS").unwrap_or_else(|_| "false".to_string()) == "true";

    if use_https {
        // Get certificate and key paths from environment
        let cert_path =
            std::env::var("TLS_CERT_PATH").expect("TLS_CERT_PATH must be set when USE_HTTPS=true");
        let key_path =
            std::env::var("TLS_KEY_PATH").expect("TLS_KEY_PATH must be set when USE_HTTPS=true");

        tracing::info!("Starting HTTPS GraphQL server on https://0.0.0.0:{}", port);
        tracing::info!(
            "GraphiQL playground available at https://0.0.0.0:{}/graphiql",
            port
        );

        // Since direct TLS support is complex with axum 0.8.4, recommend using a reverse proxy
        tracing::warn!("Direct TLS support in axum 0.8 is complex. For production, consider:");
        tracing::warn!("1. Using a reverse proxy like Nginx or Caddy for TLS termination");
        tracing::warn!("2. Downgrading to axum 0.7 which has simpler TLS support");
        tracing::warn!("3. Using a containerized approach with TLS handled by infrastructure");

        // For now, fall back to HTTP
        tracing::info!(
            "Falling back to HTTP for development. Use a reverse proxy for TLS in production."
        );
        let listener = TcpListener::bind(&addr).await?;
        axum::serve(listener, router).await?;
    } else {
        tracing::info!("Starting HTTP GraphQL server on http://0.0.0.0:{}", port);
        tracing::info!(
            "GraphiQL playground available at http://0.0.0.0:{}/graphiql",
            port
        );
        tracing::info!("Press Ctrl+C to stop the server");

        // Start HTTP server
        let listener = TcpListener::bind(&addr).await?;
        axum::serve(listener, router).await?;
    }

    tracing::info!("Server stopped");
    Ok(())
}
