use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Try to get the Supabase database URL first, fall back to DATABASE_URL
    let database_url = env::var("SUPABASE_DB_URL")
        .or_else(|_| env::var("DATABASE_URL"))
        .expect("Neither SUPABASE_DB_URL nor DATABASE_URL is set");

    println!("Attempting to connect to database...");
    println!("Using connection string: {}", database_url);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("Successfully connected to database!");

    // Test a simple query using query_scalar instead of query!
    let version: String = sqlx::query_scalar("SELECT version()")
        .fetch_one(&pool)
        .await?;

    println!("Database version: {}", version);

    Ok(())
}
