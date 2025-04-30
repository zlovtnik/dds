use async_graphql::{InputObject, ScalarType, SimpleObject, Value};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::{Decode, Encode, FromRow, Postgres, Type};
use uuid::Uuid;

/// Represents the status of a job, task, or pipeline run
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, async_graphql::Enum,
)]
#[sqlx(type_name = "status")]
pub enum Status {
    /// The entity is waiting to be processed
    Pending,
    /// The entity is currently being processed
    Running,
    /// The entity has completed successfully
    Completed,
    /// The entity has failed
    Failed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UuidScalar(pub Uuid);

#[async_graphql::Scalar]
impl ScalarType for UuidScalar {
    fn parse(value: Value) -> async_graphql::InputValueResult<Self> {
        if let Value::String(s) = value {
            Ok(UuidScalar(Uuid::parse_str(&s)?))
        } else {
            Err(async_graphql::InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_string())
    }
}

impl<'r> Decode<'r, Postgres> for UuidScalar {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let uuid = <Uuid as Decode<'r, Postgres>>::decode(value)?;
        Ok(UuidScalar(uuid))
    }
}

impl<'q> Encode<'q, Postgres> for UuidScalar {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> sqlx::encode::IsNull {
        self.0.encode_by_ref(buf)
    }
}

impl Type<Postgres> for UuidScalar {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <Uuid as Type<Postgres>>::type_info()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateTimeScalar(pub DateTime<Utc>);

#[async_graphql::Scalar]
impl ScalarType for DateTimeScalar {
    fn parse(value: Value) -> async_graphql::InputValueResult<Self> {
        if let Value::String(s) = value {
            Ok(DateTimeScalar(
                DateTime::parse_from_rfc3339(&s)?.with_timezone(&Utc),
            ))
        } else {
            Err(async_graphql::InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_rfc3339())
    }
}

impl<'r> Decode<'r, Postgres> for DateTimeScalar {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let dt = <DateTime<Utc> as Decode<'r, Postgres>>::decode(value)?;
        Ok(DateTimeScalar(dt))
    }
}

impl<'q> Encode<'q, Postgres> for DateTimeScalar {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> sqlx::encode::IsNull {
        self.0.encode_by_ref(buf)
    }
}

impl Type<Postgres> for DateTimeScalar {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <DateTime<Utc> as Type<Postgres>>::type_info()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonValueScalar(pub JsonValue);

#[async_graphql::Scalar]
impl ScalarType for JsonValueScalar {
    fn parse(value: Value) -> async_graphql::InputValueResult<Self> {
        Ok(JsonValueScalar(serde_json::to_value(value)?))
    }

    fn to_value(&self) -> Value {
        serde_json::from_value(self.0.clone()).unwrap_or(Value::Null)
    }
}

impl<'r> Decode<'r, Postgres> for JsonValueScalar {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let json = <JsonValue as Decode<'r, Postgres>>::decode(value)?;
        Ok(JsonValueScalar(json))
    }
}

impl<'q> Encode<'q, Postgres> for JsonValueScalar {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> sqlx::encode::IsNull {
        self.0.encode_by_ref(buf)
    }
}

impl Type<Postgres> for JsonValueScalar {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <JsonValue as Type<Postgres>>::type_info()
    }
}

/// Represents a job in the ETL system
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
pub struct Job {
    /// Unique identifier for the job
    pub id: UuidScalar,
    /// Name of the job
    pub name: String,
    /// Description of the job
    pub description: Option<String>,
    /// Current status of the job
    pub status: Status,
    /// When the job was created
    pub created_at: DateTimeScalar,
    /// When the job was last updated
    pub updated_at: DateTimeScalar,
}

/// Input for creating a new job
#[derive(Debug, Clone, Serialize, Deserialize, InputObject)]
pub struct CreateJob {
    /// Name of the job
    pub name: String,
    /// Description of the job
    pub description: Option<String>,
}

/// Input for updating an existing job
#[derive(Debug, Clone, Serialize, Deserialize, InputObject)]
pub struct UpdateJob {
    /// New name for the job
    pub name: Option<String>,
    /// New description for the job
    pub description: Option<String>,
    /// New status for the job
    pub status: Option<Status>,
}

/// Represents a task in the ETL system
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
pub struct Task {
    /// Unique identifier for the task
    pub id: UuidScalar,
    /// ID of the job this task belongs to
    pub job_id: UuidScalar,
    /// Name of the task
    pub name: String,
    /// Description of the task
    pub description: Option<String>,
    /// Current status of the task
    pub status: Status,
    /// Input data for the task
    pub input_data: Option<JsonValueScalar>,
    /// Output data from the task
    pub output_data: Option<JsonValueScalar>,
    /// When the task was created
    pub created_at: DateTimeScalar,
    /// When the task was last updated
    pub updated_at: DateTimeScalar,
}

/// Input for creating a new task
#[derive(Debug, Clone, Serialize, Deserialize, InputObject)]
pub struct CreateTask {
    /// ID of the job this task belongs to
    pub job_id: UuidScalar,
    /// Name of the task
    pub name: String,
    /// Description of the task
    pub description: Option<String>,
    /// Input data for the task
    pub input_data: Option<JsonValueScalar>,
}

/// Input for updating an existing task
#[derive(Debug, Clone, Serialize, Deserialize, InputObject)]
pub struct UpdateTask {
    /// New name for the task
    pub name: Option<String>,
    /// New description for the task
    pub description: Option<String>,
    /// New status for the task
    pub status: Option<Status>,
    /// New output data for the task
    pub output_data: Option<JsonValueScalar>,
    /// Error message if the task failed
    pub error_message: Option<String>,
}

/// Represents a pipeline run in the ETL system
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
pub struct PipelineRun {
    /// Unique identifier for the pipeline run
    pub id: UuidScalar,
    /// ID of the job this pipeline run belongs to
    pub job_id: UuidScalar,
    /// Current status of the pipeline run
    pub status: Status,
    /// Metrics collected during the pipeline run
    pub metrics: Option<JsonValueScalar>,
    /// When the pipeline run was created
    pub created_at: DateTimeScalar,
    /// When the pipeline run was last updated
    pub updated_at: DateTimeScalar,
}

/// Input for creating a new pipeline run
#[derive(Debug, Clone, Serialize, Deserialize, InputObject)]
pub struct CreatePipelineRun {
    /// ID of the job this pipeline run belongs to
    pub job_id: UuidScalar,
}

/// Input for updating an existing pipeline run
#[derive(Debug, Clone, Serialize, Deserialize, InputObject)]
pub struct UpdatePipelineRun {
    /// New status for the pipeline run
    pub status: Option<Status>,
    /// New metrics for the pipeline run
    pub metrics: Option<JsonValueScalar>,
    /// Error message if the pipeline run failed
    pub error_message: Option<String>,
}

