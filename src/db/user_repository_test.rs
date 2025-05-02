use crate::db::DbConnection;
use crate::models::etl::UuidScalar;
use crate::models::user::{CreateUser, UpdateUser};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

async fn setup_test_db() -> DbConnection<sqlx::Postgres> {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"))
        .await
        .expect("Failed to create test database");

    // Start a transaction that will be rolled back after the test
    let mut tx = pool.begin().await.expect("Failed to start transaction");

    // Clear the users table within the transaction
    sqlx::query("DELETE FROM users")
        .execute(&mut *tx)
        .await
        .expect("Failed to clear users table");

    // Commit the transaction
    tx.commit().await.expect("Failed to commit transaction");

    DbConnection { pool }
}

#[tokio::test]
async fn test_create_user() {
    let db = setup_test_db().await;

    let user = CreateUser {
        username: format!("testuser_{}", Uuid::new_v4()),
        email: format!("test_{}@example.com", Uuid::new_v4()),
    };

    let created = db.create_user(user).await.unwrap();
    assert!(created.username.starts_with("testuser_"));
    assert!(created.email.contains("@example.com"));
}

#[tokio::test]
async fn test_get_user() {
    let db = setup_test_db().await;

    let user = CreateUser {
        username: format!("testuser_{}", Uuid::new_v4()),
        email: format!("test_{}@example.com", Uuid::new_v4()),
    };

    let created = db.create_user(user).await.unwrap();
    let retrieved = db.get_user(created.id).await.unwrap().unwrap();

    assert_eq!(created.id.0, retrieved.id.0);
    assert_eq!(created.username, retrieved.username);
    assert_eq!(created.email, retrieved.email);
}

#[tokio::test]
async fn test_update_user() {
    let db = setup_test_db().await;

    let user = CreateUser {
        username: format!("testuser_{}", Uuid::new_v4()),
        email: format!("test_{}@example.com", Uuid::new_v4()),
    };

    let created = db.create_user(user).await.unwrap();

    let update = UpdateUser {
        username: Some(format!("updateduser_{}", Uuid::new_v4())),
        email: Some(format!("updated_{}@example.com", Uuid::new_v4())),
    };

    let updated = db.update_user(created.id, update).await.unwrap().unwrap();
    assert!(updated.username.starts_with("updateduser_"));
    assert!(updated.email.contains("updated_"));
}

#[tokio::test]
async fn test_delete_user() {
    let db = setup_test_db().await;

    let user = CreateUser {
        username: format!("testuser_{}", Uuid::new_v4()),
        email: format!("test_{}@example.com", Uuid::new_v4()),
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
