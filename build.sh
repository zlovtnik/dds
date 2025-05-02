#!/bin/bash

# Set up Rust environment
export RUSTUP_HOME=/root/.rustup
export CARGO_HOME=/root/.cargo
export PATH=$CARGO_HOME/bin:$PATH

# Source the Rust environment
. $CARGO_HOME/env

# Build the project
cargo build --bin graphql --release --target-dir=/vercel/output/target 