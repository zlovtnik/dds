#!/bin/bash

# API Configuration
export RUST_LOG=info
export RUST_BACKTRACE=1

# Use Vercel's PORT if available, otherwise default to 3000
export PORT=${PORT:-3000}

# Database Configuration (should be set via environment variables)
if [ -z "$DATABASE_URL" ]; then
    echo "Error: DATABASE_URL environment variable is not set."
    exit 1
fi

# Ensure SQLx is in online mode for production
export SQLX_OFFLINE=false

# Run the API server
# Note: In Vercel, the binary is in the current directory as 'index'
./index
