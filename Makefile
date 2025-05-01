# Rust binary name
BINARY_NAME = dds
# Build directory
BUILD_DIR = build
# Target directory
TARGET_DIR = target/release
# Vercel output directory
VERCEL_OUTPUT = .vercel/output

.PHONY: all clean build prepare-vercel

all: build prepare-vercel

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@rm -rf $(BUILD_DIR)
	@rm -rf $(VERCEL_OUTPUT)
	@cargo clean

# Build the Rust project with optimizations
build:
	@echo "Building Rust project with optimizations..."
	@RUSTFLAGS="-C opt-level=3 -C target-cpu=native" cargo build --release

# Prepare artifacts for Vercel
prepare-vercel: build
	@echo "Preparing Vercel artifacts..."
	@mkdir -p $(VERCEL_OUTPUT)/static
	@mkdir -p $(VERCEL_OUTPUT)/functions
	@cp $(TARGET_DIR)/$(BINARY_NAME) $(VERCEL_OUTPUT)/functions/
	@cp -r public/* $(VERCEL_OUTPUT)/static/ 2>/dev/null || true

# Development build
dev:
	@echo "Building development version..."
	@cargo build

# Run tests
test:
	@echo "Running tests..."
	@cargo test

# Format code
fmt:
	@echo "Formatting code..."
	@cargo fmt

# Check code
check:
	@echo "Checking code..."
	@cargo check

# Install dependencies
deps:
	@echo "Installing dependencies..."
	@cargo build

# Help command
help:
	@echo "Available commands:"
	@echo "  make all          - Build and prepare for Vercel"
	@echo "  make build        - Build the Rust project with optimizations"
	@echo "  make clean        - Clean build artifacts"
	@echo "  make dev          - Build development version"
	@echo "  make test         - Run tests"
	@echo "  make fmt          - Format code"
	@echo "  make check        - Check code"
	@echo "  make deps         - Install dependencies"
	@echo "  make prepare-vercel - Prepare artifacts for Vercel deployment" 