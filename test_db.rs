use dds::db::DbConnection;
use dotenv::dotenv;
use sqlx::Row;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv().ok();

    // Initialize database connection
    let db = DbConnection::new().await?;
    println!("Database connection established");

    // Test query to check that the database is working
    let result = sqlx::query("SELECT 1 as test").fetch_one(&db.pool).await?;
    let test_value: i32 = result.try_get("test")?;

    println!("Database test query result: {}", test_value);

    Ok(())
}
