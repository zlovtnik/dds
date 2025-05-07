#!/bin/bash

# Enable detailed logging
export RUST_LOG=debug,tower_http=debug,axum=trace,sqlx=debug
export AUTH_MOCK=true
export PORT=9090

# Check if database is configured
if [ -z "$DATABASE_URL" ]; then
  echo "WARNING: DATABASE_URL is not set. Will attempt to use .env file."
fi

# Build the project with debugging symbols
echo "Building the project in debug mode..."
cargo build || { echo "Build failed"; exit 1; }

# Start server with stdio capture
echo "Starting the API server with verbose logging on port $PORT..."
cargo run --bin graphql > server_stdout.log 2> server_stderr.log &
SERVER_PID=$!

# Wait for the server to start
echo "Waiting for server to start..."
sleep 5

# Test the health endpoint first
echo "Testing health endpoint..."
curl -v http://localhost:$PORT/health

# Test the GraphQL endpoint with a simple query
echo -e "\n\nTesting GraphQL introspection..."
curl -v -X POST http://localhost:$PORT/api/graphql \
  -H "Content-Type: application/json" \
  -d '{"query":"{__schema{queryType{name}}}"}'

# Test the login mutation
echo -e "\n\nTesting login mutation..."
curl -v -X POST http://localhost:$PORT/api/graphql \
  -H "Content-Type: application/json" \
  -d '{"operationName":"Login","query":"mutation Login($email: String!, $password: String!) {\n  login(email: $email, password: $password) {\n    token\n    refreshToken\n    user {\n      id\n      username\n      email\n    }\n  }\n}","variables":{"email":"test@example.com","password":"password123"}}'

# Check if the server is still running
if kill -0 $SERVER_PID 2>/dev/null; then
  echo -e "\n\nServer is still running. Showing logs:"
  tail -n 50 server_stderr.log
  
  # Clean up
  echo -e "\n\nStopping server..."
  kill $SERVER_PID
else
  echo -e "\n\nServer crashed! Error logs:"
  cat server_stderr.log
fi

echo "Diagnostics completed." 