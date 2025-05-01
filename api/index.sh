#!/bin/bash
cd "$(dirname "$0")"
export RUST_LOG=info
export RUST_BACKTRACE=1
export PORT=${PORT:-3000}
export SQLX_OFFLINE=false
exec ./index
