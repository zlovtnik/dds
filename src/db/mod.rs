use crate::models::user::{CreateUser, UpdateUser, User};
use chrono::{DateTime, Utc};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Database, Encode, Executor, Pool, Postgres, Type};
use std::env;
use uuid::Uuid;

/// A generic database connection wrapper that provides a connection pool and common database operations.
///
/// This struct is generic over the database type `DB` and provides type-safe database operations.
/// It requires that the database type implements the necessary traits for executing queries and handling
/// common data types like UUID, String, and DateTime.
///
/// # Type Parameters
/// * `DB` - The database type (e.g., Postgres)
/// * `DB::Connection` - The connection type for the database
/// * `Uuid` - Must be supported as a database type
/// * `String` - Must be supported as a database type
/// * `DateTime<Utc>` - Must be supported as a database type
pub struct DbConnection<DB>
where
    DB: Database,
    for<'c> &'c mut DB::Connection: Executor<'c>,
    Uuid: Type<DB> + for<'q> Encode<'q, DB>,
    String: Type<DB> + for<'q> Encode<'q, DB>,
    DateTime<Utc>: Type<DB> + for<'q> Encode<'q, DB>,
{
    /// The connection pool for managing database connections
    pub pool: Pool<DB>,
}

impl DbConnection<Postgres> {
    /// Creates a new database connection pool for PostgreSQL.
    ///
    /// # Returns
    /// * `Result<Self, sqlx::Error>` - A new `DbConnection` instance or an error if connection fails
    ///
    /// # Panics
    /// * If the `DATABASE_URL` environment variable is not set
    ///
    /// # Example
    /// ```rust
    /// let db = DbConnection::new().await?;
    /// ```
    pub async fn new() -> Result<Self, sqlx::Error> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Creates a new user in the database.
    ///
    /// # Arguments
    /// * `user` - A `CreateUser` struct containing the user's information
    ///
    /// # Returns
    /// * `Result<User, sqlx::Error>` - The created user or an error if creation fails
    ///
    /// # Example
    /// ```rust
    /// let user = db.create_user(CreateUser {
    ///     username: "john_doe".to_string(),
    ///     email: "john@example.com".to_string(),
    /// }).await?;
    /// ```
    pub async fn create_user(&self, user: CreateUser) -> Result<User, sqlx::Error> {
        let now = chrono::Utc::now();
        let id = Uuid::new_v4();

        sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, username, email, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(user.username)
        .bind(user.email)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
    }

    /// Retrieves a user from the database by their ID.
    ///
    /// # Arguments
    /// * `id` - The UUID of the user to retrieve
    ///
    /// # Returns
    /// * `Result<Option<User>, sqlx::Error>` - The user if found, None if not found, or an error
    ///
    /// # Example
    /// ```rust
    /// let user = db.get_user(user_id).await?;
    /// ```
    pub async fn get_user(&self, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    /// Updates an existing user in the database.
    ///
    /// # Arguments
    /// * `id` - The UUID of the user to update
    /// * `user` - An `UpdateUser` struct containing the fields to update
    ///
    /// # Returns
    /// * `Result<Option<User>, sqlx::Error>` - The updated user if found and updated, None if not found, or an error
    ///
    /// # Example
    /// ```rust
    /// let updated_user = db.update_user(user_id, UpdateUser {
    ///     username: Some("new_username".to_string()),
    ///     email: None,
    /// }).await?;
    /// ```
    pub async fn update_user(
        &self,
        id: Uuid,
        user: UpdateUser,
    ) -> Result<Option<User>, sqlx::Error> {
        let now = chrono::Utc::now();

        sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET 
                username = COALESCE($1, username),
                email = COALESCE($2, email),
                updated_at = $3
            WHERE id = $4
            RETURNING *
            "#,
        )
        .bind(user.username)
        .bind(user.email)
        .bind(now)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    /// Deletes a user from the database.
    ///
    /// # Arguments
    /// * `id` - The UUID of the user to delete
    ///
    /// # Returns
    /// * `Result<bool, sqlx::Error>` - True if the user was deleted, false if not found, or an error
    ///
    /// # Example
    /// ```rust
    /// let deleted = db.delete_user(user_id).await?;
    /// ```
    pub async fn delete_user(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM users WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
mod user_repository_test;
