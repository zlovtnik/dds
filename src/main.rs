mod db;
mod etl;
mod models;

use db::DbConnection;
use dotenv::dotenv;
use etl::{ETLPipeline, ETLPipelineError};
use models::user::{CreateUser, UpdateUser};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Initialize database connection
    let db = DbConnection::new().await?;

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
    let table_name = "json_data";

    // Process all JSON files in the directory
    match etl.process_directory(json_dir, table_name).await {
        Ok(_) => println!("ETL process completed successfully"),
        Err(e) => match e {
            ETLPipelineError::FileReadError(msg) => eprintln!("Error reading file: {}", msg),
            ETLPipelineError::JsonParseError(msg) => eprintln!("Error parsing JSON: {}", msg),
            ETLPipelineError::DatabaseError(e) => eprintln!("Database error: {}", e),
            ETLPipelineError::DirectoryError(msg) => eprintln!("Directory error: {}", msg),
        },
    }

    Ok(())
}
