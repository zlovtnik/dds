use anyhow::{Context, Result};
use serde_json::{Map, Value};
use sqlx::postgres::PgPool;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ETLPipelineError {
    #[error("Failed to read file: {0}")]
    FileReadError(String),

    #[error("Failed to parse JSON: {0}")]
    JsonParseError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Directory error: {0}")]
    DirectoryError(String),
}

pub struct ETLPipeline {
    pool: PgPool,
}

impl ETLPipeline {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Flattens a nested JSON object into a flat HashMap
    fn flatten_json(value: &Value, prefix: &str) -> HashMap<String, Value> {
        let mut result = HashMap::new();

        match value {
            Value::Object(map) => {
                for (key, value) in map {
                    let new_key = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}_{}", prefix, key)
                    };

                    match value {
                        Value::Object(_) => {
                            result.extend(ETLPipeline::flatten_json(value, &new_key));
                        }
                        _ => {
                            result.insert(new_key, value.clone());
                        }
                    }
                }
            }
            _ => {
                result.insert(prefix.to_string(), value.clone());
            }
        }

        result
    }

    /// Processes a single JSON file and loads it into the database
    pub async fn process_file(
        &self,
        file_path: &Path,
        table_name: &str,
    ) -> Result<(), ETLPipelineError> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| ETLPipelineError::FileReadError(format!("{:?}: {}", file_path, e)))?;

        let json_value: Value = serde_json::from_str(&content)
            .map_err(|e| ETLPipelineError::JsonParseError(format!("{:?}: {}", file_path, e)))?;

        let flattened_data = ETLPipeline::flatten_json(&json_value, "");

        // Generate dynamic SQL for insertion
        let columns: Vec<String> = flattened_data.keys().cloned().collect();
        let placeholders: Vec<String> = (1..=columns.len()).map(|i| format!("${}", i)).collect();

        let query = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table_name,
            columns.join(", "),
            placeholders.join(", ")
        );

        // Prepare the values for insertion
        let values: Vec<Value> = columns
            .iter()
            .map(|col| flattened_data[col].clone())
            .collect();

        // Execute the query
        sqlx::query(&query).bind(values).execute(&self.pool).await?;

        Ok(())
    }

    /// Processes all JSON files in a directory
    pub async fn process_directory(
        &self,
        dir_path: &Path,
        table_name: &str,
    ) -> Result<(), ETLPipelineError> {
        let entries = fs::read_dir(dir_path)
            .map_err(|e| ETLPipelineError::DirectoryError(format!("{:?}: {}", dir_path, e)))?;

        for entry in entries {
            let entry = entry.map_err(|e| ETLPipelineError::DirectoryError(format!("{:?}", e)))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                self.process_file(&path, table_name).await?;
            }
        }

        Ok(())
    }
}
