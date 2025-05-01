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
use etl::{ETLPipeline, ETLPipelineError};
use graphql::{create_router, create_schema};
use models::etl::UuidScalar;
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

    // Create a test user
    let user = db
        .create_user(CreateUser {
            username: "test_user".to_string(),
            email: "test@example.com".to_string(),
        })
        .await?;

    // Fetch the user
    let fetched_user = db.get_user(user.id).await?;
    println!("Fetched user: {:?}", fetched_user);

    // Update the user
    let update = UpdateUser {
        username: Some("updated_user".to_string()),
        email: None,
    };
    let updated_user = db.update_user(user.id, update).await?;
    println!("Updated user: {:?}", updated_user);

    // Delete the user
    let deleted = db.delete_user(user.id).await?;
    println!("User deleted: {}", deleted);

    // Create and run an ETL pipeline
    let pipeline = ETLPipeline::new(db.pool.clone());
    let data_dir = Path::new("data");
    pipeline.process_directory(data_dir).await?;

    // Start the GraphQL server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Starting GraphQL server...");
    println!("GraphQL server running on http://localhost:3000");
    println!("GraphiQL playground available at http://localhost:3000/graphiql");
    println!("Press Ctrl+C to stop the server");

    serve(listener, router).await?;

    Ok(())
}
