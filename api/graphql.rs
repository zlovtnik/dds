use async_graphql::http::GraphiQLSource;
use serde_json::json;
use std::io::{Error as IoError, ErrorKind};
use tokio::sync::broadcast;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

use dds::graphql::create_schema;

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    // Initialize database connection
    let db = dds::db::DbConnection::new()
        .await
        .map_err(|e| Error::from(Box::new(IoError::new(ErrorKind::Other, e.to_string()))))?;

    // Create event channel for GraphQL subscriptions
    let (event_sender, _) = broadcast::channel(100);

    // Create GraphQL schema
    let schema = create_schema(db.pool.clone(), event_sender);

    // Handle GraphiQL requests
    if req.uri().path() == "/graphiql" {
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .body(GraphiQLSource::build().endpoint("/graphql").finish().into())?);
    }

    // Handle GraphQL requests
    if req.uri().path() == "/graphql" {
        let body = req.body();
        let graphql_request = serde_json::from_slice::<async_graphql::Request>(&body.to_vec())
            .map_err(|e| Error::from(Box::new(IoError::new(ErrorKind::Other, e.to_string()))))?;
        let response = schema.execute(graphql_request).await;

        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(
                serde_json::to_string(&response)
                    .map_err(|e| {
                        Error::from(Box::new(IoError::new(ErrorKind::Other, e.to_string())))
                    })?
                    .into(),
            )?);
    }

    // Handle other requests
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("Not Found"))?)
}
