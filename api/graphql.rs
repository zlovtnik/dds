use axum::{routing::get, Router};
use dds::db::DbConnection;
use dds::graphql::{create_router, create_schema};
use dotenv::dotenv;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tracing;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt::init();

    // Initialize database connection
    let db = DbConnection::new().await?;
    tracing::info!("Database connection established");

    // Create event channel for GraphQL subscriptions
    let (event_sender, _) = broadcast::channel(100);
    tracing::debug!("GraphQL event channel created");

    // Create GraphQL schema and router
    let schema = create_schema(db.pool.clone(), event_sender);
    let graphql_router = create_router(schema);

    // Create the main router with the /api prefix
    let app = Router::new()
        .nest("/api", graphql_router)
        .route("/health", get(|| async { "OK" }));

    tracing::info!("Router initialized with /api prefix");

    // Start the GraphQL server
    // Default to port 8080 to match client expectations
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port).parse::<SocketAddr>()?;

    tracing::info!("Starting HTTP GraphQL server on http://{}", addr);
    tracing::info!(
        "GraphiQL playground available at http://{}/api/graphiql",
        addr
    );
    tracing::info!("Press Ctrl+C to stop the server");

    // Start HTTP server
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    tracing::info!("Server stopped");
    Ok(())
}
