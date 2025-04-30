use crate::db::DbConnection;
use crate::models::user::{CreateUser, UpdateUser};
use sqlx::postgres::PgPoolOptions;
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
    let db = setup_test_db().await;
    let user = CreateUser {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
    };

    let created_user = db.create_user(user).await.expect("Failed to create user");
    assert_eq!(created_user.username, "testuser");
    assert_eq!(created_user.email, "test@example.com");
    assert!(created_user.created_at <= chrono::Utc::now());
    assert!(created_user.updated_at <= chrono::Utc::now());
}

#[tokio::test]
async fn test_get_user() {
    let db = setup_test_db().await;
    let user = CreateUser {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
    };

    let created_user = db.create_user(user).await.expect("Failed to create user");
    let retrieved_user = db
        .get_user(created_user.id)
        .await
        .expect("Failed to get user")
        .expect("User not found");

    assert_eq!(created_user.id, retrieved_user.id);
    assert_eq!(created_user.username, retrieved_user.username);
    assert_eq!(created_user.email, retrieved_user.email);
}

#[tokio::test]
async fn test_update_user() {
    let db = setup_test_db().await;
    let user = CreateUser {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
    };

    let created_user = db.create_user(user).await.expect("Failed to create user");
    let update = UpdateUser {
        username: Some("updateduser".to_string()),
        email: None,
    };

    let updated_user = db
        .update_user(created_user.id, update)
        .await
        .expect("Failed to update user")
        .expect("User not found");

    assert_eq!(updated_user.id, created_user.id);
    assert_eq!(updated_user.username, "updateduser");
    assert_eq!(updated_user.email, created_user.email);
    assert!(updated_user.updated_at > created_user.updated_at);
}

#[tokio::test]
async fn test_delete_user() {
    let db = setup_test_db().await;
    let user = CreateUser {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
    };

    let created_user = db.create_user(user).await.expect("Failed to create user");
    let deleted = db
        .delete_user(created_user.id)
        .await
        .expect("Failed to delete user");
    assert!(deleted);

    let retrieved_user = db
        .get_user(created_user.id)
        .await
        .expect("Failed to get user");
    assert!(retrieved_user.is_none());
}

#[tokio::test]
async fn test_get_nonexistent_user() {
    let db = setup_test_db().await;
    let nonexistent_id = Uuid::new_v4();
    let retrieved_user = db
        .get_user(nonexistent_id)
        .await
        .expect("Failed to get user");
    assert!(retrieved_user.is_none());
}
