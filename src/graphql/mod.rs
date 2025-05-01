use async_graphql::{Context, Object, Schema, SimpleObject, Subscription};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::models::etl::{Job, PipelineRun, Status, Task, UuidScalar};
use crate::models::user::User;

/// GraphQL context that holds the database pool and event sender
pub struct GraphQLContext {
    pub pool: PgPool,
    pub event_sender: broadcast::Sender<ETLEvent>,
}

/// Events that can be emitted during ETL operations
#[derive(Clone, Debug, SimpleObject)]
pub struct ETLEvent {
    /// The type of event
    pub event_type: String,
    /// The ID of the entity involved
    pub entity_id: UuidScalar,
    /// The status of the entity (if applicable)
    pub status: Option<Status>,
    /// The entity data (if applicable)
    pub data: Option<String>,
}

/// Root query type for GraphQL
pub struct Query;

#[Object]
impl Query {
    /// Get a job by ID
    async fn job(&self, ctx: &Context<'_>, id: UuidScalar) -> async_graphql::Result<Option<Job>> {
        let pool = ctx.data::<GraphQLContext>()?.pool.clone();
        let job = sqlx::query_as::<_, Job>("SELECT * FROM jobs WHERE id = $1")
            .bind(id.0)
            .fetch_optional(&pool)
            .await?;
        Ok(job)
    }

    /// Get all jobs
    async fn jobs(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Job>> {
        let pool = ctx.data::<GraphQLContext>()?.pool.clone();
        let jobs = sqlx::query_as::<_, Job>("SELECT * FROM jobs ORDER BY created_at DESC")
            .fetch_all(&pool)
            .await?;
        Ok(jobs)
    }

    /// Get tasks for a job
    async fn tasks(
        &self,
        ctx: &Context<'_>,
        job_id: UuidScalar,
    ) -> async_graphql::Result<Vec<Task>> {
        let pool = ctx.data::<GraphQLContext>()?.pool.clone();
        let tasks =
            sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE job_id = $1 ORDER BY created_at")
                .bind(job_id.0)
                .fetch_all(&pool)
                .await?;
        Ok(tasks)
    }

    /// Get pipeline runs for a job
    async fn pipeline_runs(
        &self,
        ctx: &Context<'_>,
        job_id: UuidScalar,
    ) -> async_graphql::Result<Vec<PipelineRun>> {
        let pool = ctx.data::<GraphQLContext>()?.pool.clone();
        let runs = sqlx::query_as::<_, PipelineRun>(
            "SELECT * FROM pipeline_runs WHERE job_id = $1 ORDER BY created_at DESC",
        )
        .bind(job_id.0)
        .fetch_all(&pool)
        .await?;
        Ok(runs)
    }

    /// Get ETL metrics and statistics
    async fn etl_metrics(&self, ctx: &Context<'_>) -> async_graphql::Result<ETLMetrics> {
        let pool = ctx.data::<GraphQLContext>()?.pool.clone();

        // Get job statistics
        let job_stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_jobs,
                COUNT(*) FILTER (WHERE status = 'Completed') as completed_jobs,
                COUNT(*) FILTER (WHERE status = 'Failed') as failed_jobs,
                COUNT(*) FILTER (WHERE status = 'Running') as running_jobs
            FROM jobs
            "#
        )
        .fetch_one(&pool)
        .await?;

        // Get task statistics
        let task_stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_tasks,
                COUNT(*) FILTER (WHERE status = 'Completed') as completed_tasks,
                COUNT(*) FILTER (WHERE status = 'Failed') as failed_tasks,
                COUNT(*) FILTER (WHERE status = 'Running') as running_tasks
            FROM tasks
            "#
        )
        .fetch_one(&pool)
        .await?;

        Ok(ETLMetrics {
            total_jobs: job_stats.total_jobs.unwrap_or(0) as i32,
            completed_jobs: job_stats.completed_jobs.unwrap_or(0) as i32,
            failed_jobs: job_stats.failed_jobs.unwrap_or(0) as i32,
            running_jobs: job_stats.running_jobs.unwrap_or(0) as i32,
            total_tasks: task_stats.total_tasks.unwrap_or(0) as i32,
            completed_tasks: task_stats.completed_tasks.unwrap_or(0) as i32,
            failed_tasks: task_stats.failed_tasks.unwrap_or(0) as i32,
            running_tasks: task_stats.running_tasks.unwrap_or(0) as i32,
        })
    }

    /// Get a user by ID
    async fn user(&self, ctx: &Context<'_>, id: UuidScalar) -> async_graphql::Result<Option<User>> {
        let pool = ctx.data::<GraphQLContext>()?.pool.clone();
        let user = sqlx::query_as::<_, User>("SELECT * FROM public.users WHERE id = $1")
            .bind(id.0)
            .fetch_optional(&pool)
            .await?;
        Ok(user)
    }

    /// Get all users
    async fn users(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<User>> {
        let pool = ctx.data::<GraphQLContext>()?.pool.clone();
        let users = sqlx::query_as::<_, User>("SELECT * FROM public.users")
            .fetch_all(&pool)
            .await?;
        Ok(users)
    }
}

/// ETL metrics and statistics
#[derive(SimpleObject)]
pub struct ETLMetrics {
    /// Total number of jobs
    pub total_jobs: i32,
    /// Number of completed jobs
    pub completed_jobs: i32,
    /// Number of failed jobs
    pub failed_jobs: i32,
    /// Number of running jobs
    pub running_jobs: i32,
    /// Total number of tasks
    pub total_tasks: i32,
    /// Number of completed tasks
    pub completed_tasks: i32,
    /// Number of failed tasks
    pub failed_tasks: i32,
    /// Number of running tasks
    pub running_tasks: i32,
}

/// Root mutation type for GraphQL
pub struct Mutation;

#[Object]
impl Mutation {
    /// Create a new job
    async fn create_job(
        &self,
        ctx: &Context<'_>,
        name: String,
        description: Option<String>,
    ) -> async_graphql::Result<Job> {
        let pool = ctx.data::<GraphQLContext>()?.pool.clone();
        let event_sender = ctx.data::<GraphQLContext>()?.event_sender.clone();

        let job = sqlx::query_as::<_, Job>(
            r#"
            INSERT INTO jobs (id, name, description, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $5)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(name)
        .bind(description)
        .bind(Status::Pending)
        .bind(chrono::Utc::now())
        .fetch_one(&pool)
        .await?;

        // Emit event
        let _ = event_sender.send(ETLEvent {
            event_type: "JobCreated".to_string(),
            entity_id: job.id,
            status: Some(job.status),
            data: Some(serde_json::to_string(&job)?),
        });

        Ok(job)
    }

    /// Update a job's status
    async fn update_job_status(
        &self,
        ctx: &Context<'_>,
        id: UuidScalar,
        status: Status,
    ) -> async_graphql::Result<Option<Job>> {
        let pool = ctx.data::<GraphQLContext>()?.pool.clone();
        let event_sender = ctx.data::<GraphQLContext>()?.event_sender.clone();

        let job = sqlx::query_as::<_, Job>(
            r#"
            UPDATE jobs
            SET status = $1, updated_at = $2
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(status)
        .bind(chrono::Utc::now())
        .bind(id.0)
        .fetch_optional(&pool)
        .await?;

        if let Some(ref job) = job {
            // Emit event
            let _ = event_sender.send(ETLEvent {
                event_type: "JobStatusUpdated".to_string(),
                entity_id: job.id,
                status: Some(job.status),
                data: Some(serde_json::to_string(&job)?),
            });
        }

        Ok(job)
    }

    /// Create a new task
    async fn create_task(
        &self,
        ctx: &Context<'_>,
        job_id: UuidScalar,
        name: String,
        input_data: Option<serde_json::Value>,
    ) -> async_graphql::Result<Task> {
        let pool = ctx.data::<GraphQLContext>()?.pool.clone();
        let event_sender = ctx.data::<GraphQLContext>()?.event_sender.clone();

        let task = sqlx::query_as::<_, Task>(
            r#"
            INSERT INTO tasks (id, job_id, name, status, input_data, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $6)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(job_id.0)
        .bind(name)
        .bind(Status::Pending)
        .bind(input_data)
        .bind(chrono::Utc::now())
        .fetch_one(&pool)
        .await?;

        // Emit event
        let _ = event_sender.send(ETLEvent {
            event_type: "TaskCreated".to_string(),
            entity_id: task.id,
            status: Some(task.status),
            data: Some(serde_json::to_string(&task)?),
        });

        Ok(task)
    }

    /// Update a task's status
    async fn update_task_status(
        &self,
        ctx: &Context<'_>,
        id: UuidScalar,
        status: Status,
        output_data: Option<serde_json::Value>,
    ) -> async_graphql::Result<Option<Task>> {
        let pool = ctx.data::<GraphQLContext>()?.pool.clone();
        let event_sender = ctx.data::<GraphQLContext>()?.event_sender.clone();

        let task = sqlx::query_as::<_, Task>(
            r#"
            UPDATE tasks
            SET status = $1, output_data = $2, updated_at = $3
            WHERE id = $4
            RETURNING *
            "#,
        )
        .bind(status)
        .bind(output_data)
        .bind(chrono::Utc::now())
        .bind(id.0)
        .fetch_optional(&pool)
        .await?;

        if let Some(ref task) = task {
            // Emit event
            let _ = event_sender.send(ETLEvent {
                event_type: "TaskStatusUpdated".to_string(),
                entity_id: task.id,
                status: Some(task.status),
                data: Some(serde_json::to_string(&task)?),
            });
        }

        Ok(task)
    }

    /// Create a new pipeline run
    async fn create_pipeline_run(
        &self,
        ctx: &Context<'_>,
        job_id: UuidScalar,
    ) -> async_graphql::Result<PipelineRun> {
        let pool = ctx.data::<GraphQLContext>()?.pool.clone();
        let event_sender = ctx.data::<GraphQLContext>()?.event_sender.clone();

        let run = sqlx::query_as::<_, PipelineRun>(
            r#"
            INSERT INTO pipeline_runs (id, job_id, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $4)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(job_id.0)
        .bind(Status::Pending)
        .bind(chrono::Utc::now())
        .fetch_one(&pool)
        .await?;

        // Emit event
        let _ = event_sender.send(ETLEvent {
            event_type: "PipelineRunCreated".to_string(),
            entity_id: run.id,
            status: Some(run.status),
            data: Some(serde_json::to_string(&run)?),
        });

        Ok(run)
    }

    /// Update a pipeline run's status
    async fn update_pipeline_run_status(
        &self,
        ctx: &Context<'_>,
        id: UuidScalar,
        status: Status,
        metrics: Option<serde_json::Value>,
    ) -> async_graphql::Result<Option<PipelineRun>> {
        let pool = ctx.data::<GraphQLContext>()?.pool.clone();
        let event_sender = ctx.data::<GraphQLContext>()?.event_sender.clone();

        let run = sqlx::query_as::<_, PipelineRun>(
            r#"
            UPDATE pipeline_runs
            SET status = $1, metrics = $2, updated_at = $3
            WHERE id = $4
            RETURNING *
            "#,
        )
        .bind(status)
        .bind(metrics)
        .bind(chrono::Utc::now())
        .bind(id.0)
        .fetch_optional(&pool)
        .await?;

        if let Some(ref run) = run {
            // Emit event
            let _ = event_sender.send(ETLEvent {
                event_type: "PipelineRunStatusUpdated".to_string(),
                entity_id: run.id,
                status: Some(run.status),
                data: Some(serde_json::to_string(&run)?),
            });
        }

        Ok(run)
    }

    /// Create a new user
    async fn create_user(
        &self,
        ctx: &Context<'_>,
        username: String,
        email: String,
    ) -> async_graphql::Result<User> {
        let pool = ctx.data::<GraphQLContext>()?.pool.clone();
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO public.users (id, username, email, created_at, updated_at) VALUES ($1, $2, $3, NOW(), NOW()) RETURNING *",
        )
        .bind(UuidScalar(uuid::Uuid::new_v4()))
        .bind(username)
        .bind(email)
        .fetch_one(&pool)
        .await?;
        Ok(user)
    }

    /// Update an existing user
    async fn update_user(
        &self,
        ctx: &Context<'_>,
        id: UuidScalar,
        username: Option<String>,
        email: Option<String>,
    ) -> async_graphql::Result<Option<User>> {
        let pool = ctx.data::<GraphQLContext>()?.pool.clone();
        let user = sqlx::query_as::<_, User>(
            "UPDATE public.users SET username = COALESCE($1, username), email = COALESCE($2, email), updated_at = NOW() WHERE id = $3 RETURNING *",
        )
        .bind(username)
        .bind(email)
        .bind(id.0)
        .fetch_optional(&pool)
        .await?;
        Ok(user)
    }

    /// Delete a user
    async fn delete_user(&self, ctx: &Context<'_>, id: UuidScalar) -> async_graphql::Result<bool> {
        let pool = ctx.data::<GraphQLContext>()?.pool.clone();
        let result = sqlx::query("DELETE FROM public.users WHERE id = $1")
            .bind(id.0)
            .execute(&pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

/// Root subscription type for GraphQL
pub struct Subscription;

#[Subscription]
impl Subscription {
    /// Subscribe to ETL events
    async fn etl_events(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<impl futures::Stream<Item = ETLEvent>> {
        let event_sender = ctx.data::<GraphQLContext>()?.event_sender.clone();
        let mut receiver = event_sender.subscribe();

        Ok(async_stream::stream! {
            while let Ok(event) = receiver.recv().await {
                yield event;
            }
        })
    }
}

/// Create a new GraphQL schema
pub fn create_schema(
    pool: PgPool,
    event_sender: broadcast::Sender<ETLEvent>,
) -> Schema<Query, Mutation, Subscription> {
    Schema::build(Query, Mutation, Subscription)
        .data(GraphQLContext { pool, event_sender })
        .finish()
}

/// Create a new GraphQL router
pub fn create_router(schema: Schema<Query, Mutation, Subscription>) -> Router {
    Router::new()
        .route("/graphql", post(graphql_handler))
        .route("/graphiql", get(graphql_playground))
        .layer(Extension(schema))
}

/// GraphQL request handler
async fn graphql_handler(
    Extension(schema): Extension<Schema<Query, Mutation, Subscription>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let response = schema.execute(req.into_inner()).await;
    GraphQLResponse::from(response)
}

/// GraphQL playground handler
async fn graphql_playground() -> impl axum::response::IntoResponse {
    axum::response::Html(
        async_graphql::http::GraphiQLSource::build()
            .endpoint("/graphql")
            .finish(),
    )
}
