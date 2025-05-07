# Rust ETL Pipeline with PostgreSQL

A robust Rust application that combines user management functionality with an ETL (Extract, Transform, Load) pipeline for processing JSON data into PostgreSQL, featuring a GraphQL API for real-time data access.

## Features

- **User Management**
  - CRUD operations for users
  - UUID-based user identification
  - Timestamp tracking (created_at, updated_at)
  - PostgreSQL database integration

- **ETL Pipeline**
  - JSON file processing
  - Automatic flattening of nested JSON structures
  - Dynamic table creation based on JSON schema
  - PostgreSQL data loading
  - Comprehensive error handling
  - Real-time event notifications

- **GraphQL API**
  - Real-time data access
  - Subscription support for ETL events
  - Comprehensive query and mutation operations
  - Interactive GraphiQL playground
  - Job and task management
  - Pipeline run monitoring
  - ETL metrics and statistics

## Prerequisites

- Rust (latest stable version)
- PostgreSQL database
- Cargo (Rust's package manager)
- Docker (optional, for containerized deployment)
- Make (optional, for using the Makefile)

## Dependencies

- `sqlx`: Database toolkit for Rust
- `tokio`: Async runtime
- `serde`: Serialization framework
- `chrono`: Date and time handling
- `uuid`: UUID generation
- `dotenv`: Environment variable management
- `serde_json`: JSON parsing
- `anyhow`: Error handling
- `thiserror`: Custom error types
- `async-graphql`: GraphQL implementation
- `async-graphql-axum`: Axum integration for GraphQL
- `axum`: Web framework
- `broadcast`: Event broadcasting

## Project Structure

```
.
├── src/
│   ├── main.rs           # Application entry point
│   ├── etl.rs            # ETL pipeline implementation
│   ├── graphql/          # GraphQL implementation
│   │   ├── mod.rs        # GraphQL schema and resolvers
│   │   └── types.rs      # GraphQL types and models
│   ├── models/
│   │   ├── mod.rs
│   │   └── user.rs       # User model definitions
│   └── db/
│       ├── mod.rs        # Database connection and operations
│       └── user_repository_test.rs
├── migrations/           # Database migrations
├── Dockerfile           # Docker configuration
├── Makefile             # Build and run shortcuts
├── .github/
│   └── workflows/        # GitHub Actions workflows
└── Cargo.toml           # Project dependencies
```

## Setup

1. Clone the reposit
2. Create a `.env` file with your database configuration:
   ```
   DATABASE_URL=postgres://username:password@localhost:5432/database_name
   ```
3. Run database migrations:
   ```bash
   make migrate-up
   ```
   or
   ```bash
   sqlx migrate run
   ```
4. Build the project:
   ```bash
   make build
   ```
   or
   ```bash
   cargo build --release --bin dds
   ```

## Using the Makefile

This project includes a Makefile to simplify common operations:

```bash
# Build the application
make build

# Run the application
make run

# Run tests
make test

# Build Docker image
make docker-build

# Run Docker container
make docker-run

# Run database migrations
make migrate-up

# Run in development mode
make dev
```

Run `make help` to see all available commands.

## Continuous Integration

This project uses GitHub Actions for continuous integration and deployment. The workflow includes:

- Building the Rust application
- Running tests
- Building a Docker image (on merge to main)

The workflow configuration is located in `.github/workflows/rust.yml`.

## Usage

### GraphQL API

The application provides a GraphQL API at `http://0.0.0.0:4040/graphql` with an interactive playground available at `http://0.0.0.0:4040/graphiql`.

Key GraphQL features:
- Query jobs, tasks, and pipeline runs
- Create and manage ETL jobs
- Monitor job status and metrics
- Subscribe to real-time ETL events
- View ETL statistics and metrics

### User Management

The application provides a complete CRUD interface for user management:

```rust
// Create a user
let new_user = CreateUser {
    username: "testuser".to_string(),
    email: "test@example.com".to_string(),
};
let user = db.create_user(new_user).await?;

// Get a user
let fetched_user = db.get_user(user.id).await?;

// Update a user
let update = UpdateUser {
    username: Some("updateduser".to_string()),
    email: None,
};
let updated_user = db.update_user(user.id, update).await?;

// Delete a user
let deleted = db.delete_user(user.id).await?;
```

### ETL Pipeline

To process JSON files:

1. Create a directory `data/json` in your project root
2. Place JSON files in this directory
3. Run the application

The ETL pipeline will:
- Read all JSON files from the directory
- Flatten nested JSON structures
- Create appropriate database tables
- Load the data into PostgreSQL
- Emit real-time events for monitoring

Example JSON file:
```json
{
  "user": {
    "name": "John Doe",
    "address": {
      "street": "123 Main St",
      "city": "Anytown"
    }
  }
}
```

This will be flattened into:
```json
{
  "user_name": "John Doe",
  "user_address_street": "123 Main St",
  "user_address_city": "Anytown"
}
```

## Error Handling

The application uses a comprehensive error handling system:

- Custom error types for different failure scenarios
- Detailed error messages with context
- Proper error propagation
- Graceful error recovery

Error types include:
- `FileReadError`: File reading issues
- `JsonParseError`: JSON parsing errors
- `DatabaseError`: Database-related errors
- `DirectoryError`: Directory-related errors

## Testing

Run the test suite:
```bash
cargo test
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details. 