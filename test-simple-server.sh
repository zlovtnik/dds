#!/bin/bash

# Enable detailed logging
export RUST_LOG=debug,tower_http=debug,axum=trace
export PORT=9090

# Build and run the simple server
echo "Building and starting the simple server on port $PORT..."
cargo run --bin simple_server > server_stdout.log 2> server_stderr.log &
SERVER_PID=$!

# Wait for the server to start
echo "Waiting for server to start..."
sleep 3

# Test the health endpoint
echo "Testing health endpoint..."
curl -v http://localhost:$PORT/health

# Test the root endpoint
echo -e "\n\nTesting root endpoint..."
curl -v http://localhost:$PORT/

# Check if the server is still running
if kill -0 $SERVER_PID 2>/dev/null; then
  echo -e "\n\nServer is running correctly! Showing logs:"
  tail -n 20 server_stderr.log
  
  # Clean up
  echo -e "\n\nStopping server..."
  kill $SERVER_PID
else
  echo -e "\n\nServer crashed! Error logs:"
  cat server_stderr.log
fi

echo "Test completed." 