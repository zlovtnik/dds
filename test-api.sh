#!/bin/bash

# Make sure we're using the development environment
export AUTH_MOCK=true
export PORT=9090

# Build the project
echo "Building the project..."
cargo build || { echo "Build failed"; exit 1; }

# Start the GraphQL API server in the background
echo "Starting the API server on port $PORT..."
cargo run --bin graphql &
SERVER_PID=$!

# Wait for the server to start
echo "Waiting for server to start..."
sleep 2

# Test the login endpoint
echo "Testing login mutation..."
curl -X POST http://localhost:$PORT/api/graphql \
  -H "Content-Type: application/json" \
  -d '{"operationName":"Login","query":"mutation Login($email: String!, $password: String!) {\n  login(email: $email, password: $password) {\n    token\n    refreshToken\n    user {\n      id\n      username\n      email\n      __typename\n    }\n    __typename\n  }\n}","variables":{"email":"test@example.com","password":"password123"}}'

echo -e "\n\nServer logs:"
# Give some time for logs to appear
sleep 1

# Cleanup - kill the server
kill $SERVER_PID
echo "Test completed." 