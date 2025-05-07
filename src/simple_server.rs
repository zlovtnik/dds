use axum::{routing::get, Router};
use dotenv::dotenv;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Simple router with just a health check
    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/", get(|| async { "GraphQL API Server" }));

    tracing::info!("Router initialized with basic routes");

    // Start the server
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port).parse::<SocketAddr>()?;

    tracing::info!("Starting simple test server on http://{}", addr);

    // Start HTTP server
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Server listening on {}", addr);
    axum::serve(listener, app).await?;

    tracing::info!("Server stopped");
    Ok(())
}
