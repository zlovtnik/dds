#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

echo "--- Starting Vercel Rust Build ---"

# Install Rust if not already installed
if ! command -v rustc &> /dev/null; then
    echo "Installing Rust..."
    # Use --no-modify-path as we will source manually
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
    # Source the cargo environment script manually to update PATH for this script session
    source "$HOME/.cargo/env"
else
    echo "Rust already installed."
    # Ensure PATH includes cargo bin if Rust was already present (e.g., from cache)
    # Adding it explicitly is safe even if already there.
    export PATH="$HOME/.cargo/bin:$PATH"
fi

echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"

# Install sqlx-cli
echo "Installing sqlx-cli..."
cargo install sqlx-cli
echo "sqlx-cli installed."

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
echo ">>> Running cargo sqlx prepare command <<<"
set -x # Start detailed command logging just before the potentially slow command
cargo sqlx prepare --workspace -- --all-targets || cargo sqlx prepare
set +x # Stop detailed command logging
echo ">>> cargo sqlx prepare command finished <<<"
echo "SQLx preparation complete."

# --- Build ---
# Set SQLX_OFFLINE=true for the final build
export SQLX_OFFLINE=true
echo "Building release binary (SQLX_OFFLINE=$SQLX_OFFLINE)..."
# Build with release profile and API features
cargo build --release --features api
echo "Build complete."

# --- Prepare Vercel Output ---
# Create the standard Vercel API output directory
OUTPUT_DIR="api"
mkdir -p "$OUTPUT_DIR"
echo "Created output directory: $OUTPUT_DIR"

# Copy the compiled binary to the output directory
cp target/release/dds "$OUTPUT_DIR/index"
echo "Copied binary to $OUTPUT_DIR/index"

# Make the binary executable
chmod +x "$OUTPUT_DIR/index"

# Create a simple launcher script
cat > "$OUTPUT_DIR/index.sh" << 'EOF'
#!/bin/bash
cd "$(dirname "$0")"
export RUST_LOG=info
export RUST_BACKTRACE=1
export PORT=${PORT:-3000}
export SQLX_OFFLINE=false
exec ./index
EOF

# Make the launcher script executable
chmod +x "$OUTPUT_DIR/index.sh"

echo "--- Vercel Rust Build Finished Successfully ---"