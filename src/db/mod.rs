use crate::models::user::{CreateUser, UpdateUser, User};
use chrono::{DateTime, Utc};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Database, Encode, Executor, Pool, Postgres, Sqlite, Type};
use std::env;
use uuid::Uuid;

pub struct DbConnection<DB>
where
    DB: Database,
    for<'c> &'c mut DB::Connection: Executor<'c>,
    Uuid: Type<DB> + for<'q> Encode<'q, DB>,
    String: Type<DB> + for<'q> Encode<'q, DB>,
    DateTime<Utc>: Type<DB> + for<'q> Encode<'q, DB>,
{
    pub pool: Pool<DB>,
}

impl DbConnection<Postgres> {
    pub async fn new() -> Result<Self, sqlx::Error> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;

        Ok(Self { pool })
    }

    pub async fn create_user(&self, user: CreateUser) -> Result<User, sqlx::Error> {
        let now = chrono::Utc::now();
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO users (id, username, email, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(id)
        .bind(user.username)
        .bind(user.email)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        self.get_user(id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)
    }

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

    pub async fn update_user(
        &self,
        id: Uuid,
        user: UpdateUser,
    ) -> Result<Option<User>, sqlx::Error> {
        let now = chrono::Utc::now();

        sqlx::query(
            r#"
            UPDATE users
            SET 
                username = COALESCE($1, username),
                email = COALESCE($2, email),
                updated_at = $3
            WHERE id = $4
            "#,
        )
        .bind(user.username)
        .bind(user.email)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.get_user(id).await
    }

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

impl DbConnection<Sqlite> {
    pub async fn create_user(&self, user: CreateUser) -> Result<User, sqlx::Error> {
        let now = chrono::Utc::now();
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO users (id, username, email, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(id)
        .bind(user.username)
        .bind(user.email)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        self.get_user(id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)
    }

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

    pub async fn update_user(
        &self,
        id: Uuid,
        user: UpdateUser,
    ) -> Result<Option<User>, sqlx::Error> {
        let now = chrono::Utc::now();

        sqlx::query(
            r#"
            UPDATE users
            SET 
                username = COALESCE($1, username),
                email = COALESCE($2, email),
                updated_at = $3
            WHERE id = $4
            "#,
        )
        .bind(user.username)
        .bind(user.email)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.get_user(id).await
    }

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
