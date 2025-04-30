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
use etl::{ETLPipeline, ETLPipelineError};
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
    dotenv().ok();

    // Initialize database connection
    let db = DbConnection::new().await?;

    // Create event channel for GraphQL subscriptions
    let (event_sender, _) = broadcast::channel(100);

    // Create GraphQL schema and router
    let schema = create_schema(db.pool.clone(), event_sender);
    let router = create_router(schema);

    // Start the GraphQL server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4040").await?;
    let server = serve(listener, router.into_make_service());

    println!("GraphQL server running on http://0.0.0.0:4040");
    println!("GraphiQL playground available at http://0.0.0.0:4040/graphiql");

    // Run the server in the background
    tokio::spawn(async move {
        if let Err(e) = server.await {
            eprintln!("Server error: {}", e);
        }
    });

    // Example usage of CRUD operations
    let new_user = CreateUser {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
    };

    // Create a user
    let user = db.create_user(new_user).await?;
    println!("Created user: {:?}", user);

    // Get a user
    let fetched_user = db.get_user(user.id).await?;
    println!("Fetched user: {:?}", fetched_user);

    // Update a user
    let update = UpdateUser {
        username: Some("updateduser".to_string()),
        email: None,
    };
    let updated_user = db.update_user(user.id, update).await?;
    println!("Updated user: {:?}", updated_user);

    // Delete a user
    let deleted = db.delete_user(user.id).await?;
    println!("User deleted: {}", deleted);

    // Example of ETL pipeline usage
    let etl = ETLPipeline::new(db.pool);
    let json_dir = Path::new("data/json");

    // Process all JSON files in the directory
    match etl.process_directory(json_dir).await {
        Ok(_) => println!("ETL process completed successfully"),
        Err(e) => match e {
            ETLPipelineError::FileReadError(msg) => eprintln!("Error reading file: {}", msg),
            ETLPipelineError::JsonParseError(msg) => eprintln!("Error parsing JSON: {}", msg),
            ETLPipelineError::DatabaseError(e) => eprintln!("Database error: {}", e),
            ETLPipelineError::DirectoryError(msg) => eprintln!("Directory error: {}", msg),
        },
    }

    // Keep the main thread alive
    tokio::signal::ctrl_c().await?;
    println!("Shutting down...");

    Ok(())
}
