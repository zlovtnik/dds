#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

echo "--- Starting Vercel Rust Build ---"

# Install Rust if not already installed
if ! command -v rustc &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
else
    echo "Rust already installed."
fi

echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"

# --- SQLx Preparation ---
# Ensure DATABASE_URL is set in Vercel Environment Variables
if [ -z "$DATABASE_URL" ]; then
  echo "Error: DATABASE_URL environment variable is not set."
  exit 1
fi
echo "DATABASE_URL is set."

# Set SQLX_OFFLINE=false for preparation
export SQLX_OFFLINE=false
echo "Preparing SQLx (SQLX_OFFLINE=$SQLX_OFFLINE)..."
# Consider adding --check to avoid full recompilation if possible,
# but full prepare is safer in CI.
cargo sqlx prepare --workspace -- --all-targets || cargo sqlx prepare # Try workspace first, fallback if not a workspace
echo "SQLx preparation complete."

# --- Build ---
# Set SQLX_OFFLINE=true for the final build
export SQLX_OFFLINE=true
echo "Building release binary (SQLX_OFFLINE=$SQLX_OFFLINE)..."
cargo build --release
echo "Build complete."

# --- Prepare Vercel Output ---
# Create the standard Vercel API output directory
OUTPUT_DIR="api"
mkdir -p "$OUTPUT_DIR"
echo "Created output directory: $OUTPUT_DIR"

# Copy the compiled binary to the output directory.
# IMPORTANT: Rename the binary if you want a specific endpoint.
# - 'index': Responds to requests at /api/
# - 'my_function': Responds to requests at /api/my_function
# We'll use 'index' here for the root API endpoint.
cp target/release/dds "$OUTPUT_DIR/index"
echo "Copied binary to $OUTPUT_DIR/index"

# Make the binary executable (redundant usually, but safe)
chmod +x "$OUTPUT_DIR/index"

echo "--- Vercel Rust Build Finished Successfully ---"

# No need for .func or launcher.sh with this approach
# Vercel's 'provided' runtime will execute api/index directly.