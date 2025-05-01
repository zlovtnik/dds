/// The main module of the application.
///
/// This module contains the entry point of the application and demonstrates the usage of
/// the database operations and ETL pipeline functionality.
mod db;
mod etl;
mod graphql;
mod models;

use axum::serve;
use db::DbConnection;
use dotenv::dotenv;
use env_logger;
use etl::ETLPipeline;
use graphql::{create_router, create_schema};
use models::user::{CreateUser, UpdateUser};
use std::path::Path;
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
    env_logger::init();
    dotenv().ok();

    // Initialize database connection
    let db = DbConnection::new().await?;

    // Create event channel for GraphQL subscriptions
    let (event_sender, _) = broadcast::channel(100);

    // Create GraphQL schema and router
    let schema = create_schema(db.pool.clone(), event_sender);
    let router = create_router(schema);

    // Start the GraphQL server
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("Starting GraphQL server...");
    println!("GraphQL server running on http://localhost:{}", port);
    println!(
        "GraphiQL playground available at http://localhost:{}/graphiql",
        port
    );
    println!("Press Ctrl+C to stop the server");

    serve(listener, router).await?;

    Ok(())
}
