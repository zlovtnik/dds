.PHONY: build run test clean docker-build docker-run migrate-up dev help

# Default target
help:
	@echo "Available targets:"
	@echo "  build         - Build the application in release mode"
	@echo "  run           - Run the application"
	@echo "  test          - Run tests"
	@echo "  clean         - Clean build artifacts"
	@echo "  docker-build  - Build Docker image"
	@echo "  docker-run    - Run application in Docker container"
	@echo "  migrate-up    - Run database migrations"
	@echo "  dev           - Run in development mode with auto-reload"

# Build the application
build:
	cargo build --release --bin dds

# Run the application
run:
	cargo run --bin dds

# Run tests
test:
	cargo test

# Clean build artifacts
clean:
	cargo clean

# Build Docker image
docker-build:
	docker build -t dds:latest .

# Run Docker container
docker-run:
	docker run -p 3000:3000 --env-file .env dds:latest

# Run database migrations
migrate-up:
	sqlx migrate run

# Run in development mode
dev:
	RUST_LOG=debug cargo run --bin dds 