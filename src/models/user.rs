use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::models::etl::{DateTimeScalar, UuidScalar};

/// Represents a user in the system.
///
/// This struct is used to represent a user entity in the database and includes all user-related information.
/// It implements `Serialize`, `Deserialize`, and `FromRow` for JSON serialization and database row mapping.
#[derive(Debug, Serialize, Deserialize, FromRow, async_graphql::SimpleObject)]
pub struct User {
    /// The unique identifier for the user
    pub id: UuidScalar,
    /// The username chosen by the user
    pub username: String,
    /// The email address of the user
    pub email: String,
    /// The timestamp when the user was created
    pub created_at: DateTimeScalar,
    /// The timestamp when the user was last updated
    pub updated_at: DateTimeScalar,
}

/// Represents the data needed to create a new user.
///
/// This struct is used when creating a new user and contains only the required fields.
/// It implements `Serialize` and `Deserialize` for JSON serialization.
#[derive(Debug, Serialize, Deserialize, async_graphql::InputObject)]
pub struct CreateUser {
    /// The username for the new user
    pub username: String,
    /// The email address for the new user
    pub email: String,
}

/// Represents the data that can be updated for an existing user.
///
/// This struct is used when updating an existing user and contains optional fields.
/// Fields that are `None` will not be updated in the database.
/// It implements `Serialize` and `Deserialize` for JSON serialization.
#[derive(Debug, Serialize, Deserialize, async_graphql::InputObject)]
pub struct UpdateUser {
    /// The new username (if provided)
    pub username: Option<String>,
    /// The new email address (if provided)
    pub email: Option<String>,
}
