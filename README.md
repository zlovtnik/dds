# Rust ETL Pipeline with PostgreSQL

A robust Rust application that combines user management functionality with an ETL (Extract, Transform, Load) pipeline for processing JSON data into PostgreSQL.

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

## Prerequisites

- Rust (latest stable version)
- PostgreSQL database
- Cargo (Rust's package manager)

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

## Project Structure

```
.
├── src/
│   ├── main.rs           # Application entry point
│   ├── etl.rs            # ETL pipeline implementation
│   ├── models/
│   │   ├── mod.rs
│   │   └── user.rs       # User model definitions
│   └── db/
│       ├── mod.rs        # Database connection and operations
│       └── user_repository_test.rs
├── migrations/           # Database migrations
└── Cargo.toml           # Project dependencies
```

## Setup

1. Clone the repository
2. Create a `.env` file with your database configuration:
   ```
   DATABASE_URL=postgres://username:password@localhost:5432/database_name
   ```
3. Run database migrations:
   ```bash
   sqlx migrate run
   ```
4. Build the project:
   ```bash
   cargo build
   ```

## Usage

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