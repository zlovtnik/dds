#!/bin/bash

# Install Rust if not already installed
if ! command -v rustc &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

# Set environment variables for SQLx
export DATABASE_URL="postgres://dummy:dummy@localhost:5432/dummy"
export SQLX_OFFLINE=false

# Build the Rust application
cargo build --release

# Create the output directory
mkdir -p .vercel/output/functions

# Copy the binary to the output directory
cp target/release/dds .vercel/output/functions/

# Create the function configuration
cat > .vercel/output/functions/main.func << EOF
{
    "runtime": "provided",
    "handler": "dds",
    "launcherType": "bash",
    "environment": {
        "PORT": "0",
        "SQLX_OFFLINE": "false",
        "SUPABASE_URL": "\${SUPABASE_URL}",
        "SUPABASE_KEY": "\${SUPABASE_KEY}",
        "SUPABASE_DB_URL": "\${SUPABASE_DB_URL}"
    }
}
EOF 