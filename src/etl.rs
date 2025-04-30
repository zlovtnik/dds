use anyhow::Result;
use serde_json::Value;
use sqlx::postgres::PgPool;
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Error types that can occur during ETL pipeline operations.
///
/// This enum represents various errors that can occur during the Extract, Transform, Load process.
#[derive(Error, Debug)]
pub enum ETLPipelineError {
    /// Error occurred while reading a file
    #[error("Failed to read file: {0}")]
    FileReadError(String),

    /// Error occurred while parsing JSON data
    #[error("Failed to parse JSON: {0}")]
    JsonParseError(String),

    /// Error occurred during database operations
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    /// Error occurred while processing a directory
    #[error("Directory error: {0}")]
    DirectoryError(String),
}

/// A pipeline for Extract, Transform, Load (ETL) operations.
///
/// This struct provides functionality to process JSON files and load them into a PostgreSQL database.
pub struct ETLPipeline {
    /// The PostgreSQL connection pool used for database operations
    pool: PgPool,
}

impl ETLPipeline {
    /// Creates a new ETL pipeline instance.
    ///
    /// # Arguments
    /// * `pool` - A PostgreSQL connection pool
    ///
    /// # Returns
    /// A new `ETLPipeline` instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Processes a single JSON file and loads it into the database.
    ///
    /// This method reads a JSON file, parses its contents, and stores both the file name
    /// and the JSON data in the database.
    ///
    /// # Arguments
    /// * `file_path` - The path to the JSON file to process
    ///
    /// # Returns
    /// * `Result<(), ETLPipelineError>` - Ok(()) if successful, or an error if processing fails
    ///
    /// # Errors
    /// * `FileReadError` - If the file cannot be read
    /// * `JsonParseError` - If the JSON content cannot be parsed
    /// * `DatabaseError` - If the database operation fails
    pub async fn process_file(&self, file_path: &Path) -> Result<(), ETLPipelineError> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| ETLPipelineError::FileReadError(format!("{:?}: {}", file_path, e)))?;

        let json_value: Value = serde_json::from_str(&content)
            .map_err(|e| ETLPipelineError::JsonParseError(format!("{:?}: {}", file_path, e)))?;

        // Get the file name
        let file_name = file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Insert the JSON data into the table
        sqlx::query(
            r#"
            INSERT INTO json_data (file_name, data)
            VALUES ($1, $2)
            "#,
        )
        .bind(file_name)
        .bind(json_value)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Processes all JSON files in a directory.
    ///
    /// This method scans a directory for JSON files and processes each one using `process_file`.
    ///
    /// # Arguments
    /// * `dir_path` - The path to the directory containing JSON files
    ///
    /// # Returns
    /// * `Result<(), ETLPipelineError>` - Ok(()) if successful, or an error if processing fails
    ///
    /// # Errors
    /// * `DirectoryError` - If the directory cannot be read
    /// * Any error from `process_file` if file processing fails
    pub async fn process_directory(&self, dir_path: &Path) -> Result<(), ETLPipelineError> {
        let entries = fs::read_dir(dir_path)
            .map_err(|e| ETLPipelineError::DirectoryError(format!("{:?}: {}", dir_path, e)))?;

        for entry in entries {
            let entry = entry.map_err(|e| ETLPipelineError::DirectoryError(format!("{:?}", e)))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                self.process_file(&path).await?;
            }
        }

        Ok(())
    }
}
