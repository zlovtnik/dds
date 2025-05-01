use crate::db::DbConnection;
use crate::models::etl::UuidScalar;
use crate::models::user::{CreateUser, UpdateUser, User};
use chrono::Utc;
use sqlx::postgres::{PgPoolOptions, Postgres};
use uuid::Uuid;

async fn setup_test_db() -> DbConnection<sqlx::Postgres> {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect("postgres://postgres:postgres@localhost:5432/test_db")
        .await
        .expect("Failed to create test database");

    // Start a transaction
    let mut tx = pool.begin().await.expect("Failed to start transaction");

    // Clear the users table
    let _ = sqlx::query("DELETE FROM users")
        .execute(&mut *tx)
        .await
        .expect("Failed to clear users table");

    // Commit the transaction
    let _ = tx.commit().await.expect("Failed to commit transaction");

    DbConnection { pool }
}

#[tokio::test]
async fn test_create_user() {
    let db = DbConnection::<Postgres>::new().await.unwrap();

    let user = CreateUser {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
    };

    let created = db.create_user(user).await.unwrap();
    assert_eq!(created.username, "testuser");
    assert_eq!(created.email, "test@example.com");
}

#[tokio::test]
async fn test_get_user() {
    let db = DbConnection::<Postgres>::new().await.unwrap();

    let user = CreateUser {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
    };

    let created = db.create_user(user).await.unwrap();
    let retrieved = db.get_user(created.id).await.unwrap().unwrap();

    assert_eq!(created.id.0, retrieved.id.0);
    assert_eq!(created.username, retrieved.username);
    assert_eq!(created.email, retrieved.email);
}

#[tokio::test]
async fn test_update_user() {
    let db = DbConnection::<Postgres>::new().await.unwrap();

    let user = CreateUser {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
    };

    let created = db.create_user(user).await.unwrap();

    let update = UpdateUser {
        username: Some("updateduser".to_string()),
        email: Some("updated@example.com".to_string()),
    };

    let updated = db.update_user(created.id, update).await.unwrap().unwrap();
    assert_eq!(updated.username, "updateduser");
    assert_eq!(updated.email, "updated@example.com");
}

#[tokio::test]
async fn test_delete_user() {
    let db = DbConnection::<Postgres>::new().await.unwrap();

    let user = CreateUser {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
    };

    let created = db.create_user(user).await.unwrap();
    let deleted = db.delete_user(created.id).await.unwrap();
    assert!(deleted);

    let retrieved = db.get_user(created.id).await.unwrap();
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_get_nonexistent_user() {
    let db = setup_test_db().await;
    let nonexistent_id = UuidScalar(Uuid::new_v4());
    let retrieved_user = db
        .get_user(nonexistent_id)
        .await
        .expect("Failed to get user");
    assert!(retrieved_user.is_none());
}
