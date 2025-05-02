use crate::models::user::{CreateUser, UpdateUser, User};
use chrono::{DateTime, Utc};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Database, Encode, Executor, Pool, Postgres, Type};
use std::env;
use uuid::Uuid;

use crate::models::etl::UuidScalar;

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
    /// ```no_run
    /// use dds::db::DbConnection;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let db = DbConnection::new().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new() -> Result<Self, sqlx::Error> {
        println!("Environment variables:");
        for (key, value) in env::vars() {
            println!("{}: {}", key, value);
        }

        // Try to get the Supabase database URL first, fall back to DATABASE_URL
        let database_url = env::var("SUPABASE_DB_URL")
            .or_else(|_| env::var("DATABASE_URL"))
            .expect("Neither SUPABASE_DB_URL nor DATABASE_URL is set");

        println!("Using database URL: {}", database_url);

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Creates a new user in the database.
    ///
    /// # Arguments
    /// * `user` - The user data to create
    ///
    /// # Returns
    /// * `Result<User, sqlx::Error>` - The created user or an error if creation fails
    ///
    /// # Example
    /// ```no_run
    /// use dds::db::DbConnection;
    /// use dds::models::user::CreateUser;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let db = DbConnection::new().await?;
    ///     let user = CreateUser {
    ///         username: "johndoe".to_string(),
    ///         email: "john@example.com".to_string(),
    ///     };
    ///     let created_user = db.create_user(user).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn create_user(&self, user: CreateUser) -> Result<User, sqlx::Error> {
        let query = "INSERT INTO public.users (id, username, email, created_at, updated_at) VALUES ($1, $2, $3, NOW(), NOW()) RETURNING *";
        println!("Executing SQL query: {}", query);
        let user = sqlx::query_as::<_, User>(query)
            .bind(UuidScalar(Uuid::new_v4()))
            .bind(user.username)
            .bind(user.email)
            .fetch_one(&self.pool)
            .await?;

        Ok(user)
    }

    /// Retrieves a user from the database by their ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the user to retrieve
    ///
    /// # Returns
    /// * `Result<Option<User>, sqlx::Error>` - The user if found, None if not found, or an error
    ///
    /// # Example
    /// ```no_run
    /// use dds::db::DbConnection;
    /// use dds::models::etl::UuidScalar;
    /// use uuid::Uuid;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let db = DbConnection::new().await?;
    ///     let user_id = UuidScalar(Uuid::new_v4());
    ///     let user = db.get_user(user_id).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_user(&self, id: UuidScalar) -> Result<Option<User>, sqlx::Error> {
        let query = "SELECT * FROM public.users WHERE id = $1";
        println!("Executing SQL query: {}", query);
        let user = sqlx::query_as::<_, User>(query)
            .bind(id.0)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    /// Updates a user in the database.
    ///
    /// # Arguments
    /// * `id` - The ID of the user to update
    /// * `user` - The user data to update
    ///
    /// # Returns
    /// * `Result<Option<User>, sqlx::Error>` - The updated user if found, None if not found, or an error
    ///
    /// # Example
    /// ```no_run
    /// use dds::db::DbConnection;
    /// use dds::models::user::UpdateUser;
    /// use dds::models::etl::UuidScalar;
    /// use uuid::Uuid;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let db = DbConnection::new().await?;
    ///     let user_id = UuidScalar(Uuid::new_v4());
    ///     let update = UpdateUser {
    ///         username: Some("newusername".to_string()),
    ///         email: None,
    ///     };
    ///     let updated_user = db.update_user(user_id, update).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn update_user(
        &self,
        id: UuidScalar,
        user: UpdateUser,
    ) -> Result<Option<User>, sqlx::Error> {
        let query = "UPDATE public.users SET username = COALESCE($1, username), email = COALESCE($2, email), updated_at = NOW() WHERE id = $3 RETURNING *";
        println!("Executing SQL query: {}", query);
        let user = sqlx::query_as::<_, User>(query)
            .bind(user.username)
            .bind(user.email)
            .bind(id.0)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    /// Deletes a user from the database.
    ///
    /// # Arguments
    /// * `id` - The ID of the user to delete
    ///
    /// # Returns
    /// * `Result<bool, sqlx::Error>` - True if the user was deleted, False if not found, or an error
    ///
    /// # Example
    /// ```no_run
    /// use dds::db::DbConnection;
    /// use dds::models::etl::UuidScalar;
    /// use uuid::Uuid;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let db = DbConnection::new().await?;
    ///     let user_id = UuidScalar(Uuid::new_v4());
    ///     let deleted = db.delete_user(user_id).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn delete_user(&self, id: UuidScalar) -> Result<bool, sqlx::Error> {
        let query = "DELETE FROM public.users WHERE id = $1";
        println!("Executing SQL query: {}", query);
        let result = sqlx::query(query).bind(id.0).execute(&self.pool).await?;

        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
mod user_repository_test;
