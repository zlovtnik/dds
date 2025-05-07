# Build stage
FROM rust:1.83-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /usr/src/dds

# Copy only Cargo files first to leverage Docker cache
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

# Set environment variables
ENV RUST_LOG=debug
ENV LOG_DIR=/app/logs
ENV DATABASE_URL=postgres://postgres.anymjmnmbhpmkzqmnrwd:gHIwiVXsSvg2RrQv@aws-0-us-east-1.pooler.supabase.com:5432/postgres?sslmode=require
ENV SUPABASE_DB_URL=postgres://postgres.anymjmnmbhpmkzqmnrwd:gHIwiVXsSvg2RrQv@aws-0-us-east-1.pooler.supabase.com:5432/postgres?sslmode=require

# Create logs directory
RUN mkdir -p /app/logs

# Create necessary directories and mock files for the build
RUN mkdir -p api && touch api/graphql.rs && \
    touch test_db.rs

# Build only the main binary with --bin dds
RUN cargo build --release --bin dds

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 dds

# Set working directory
WORKDIR /app

# Create logs directory
RUN mkdir -p /app/logs

# Copy only the necessary files from builder
COPY --from=builder /usr/src/dds/target/release/dds /app/dds
COPY --from=builder /usr/src/dds/migrations /app/migrations

# Set ownership
RUN chown -R dds:dds /app

# Switch to non-root user
USER dds

# Set environment variables
ENV RUST_LOG=debug
ENV LOG_DIR=/app/logs
ENV DATABASE_URL=postgres://postgres.anymjmnmbhpmkzqmnrwd:gHIwiVXsSvg2RrQv@aws-0-us-east-1.pooler.supabase.com:5432/postgres?sslmode=require
ENV SUPABASE_DB_URL=postgres://postgres.anymjmnmbhpmkzqmnrwd:gHIwiVXsSvg2RrQv@aws-0-us-east-1.pooler.supabase.com:5432/postgres?sslmode=require

# Expose ports
EXPOSE 3000

# Start the application
# Main app is "./dds" (GraphQL API server)
CMD ["./dds"] 