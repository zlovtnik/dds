use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PerUser {
    #[sqlx(rename = "USER_ID")]
    pub user_id: i64,
    #[sqlx(rename = "BUSINESS_GROUP_ID")]
    pub business_group_id: i64,
    #[sqlx(rename = "ACTIVE_FLAG")]
    pub active_flag: String, // VARCHAR2(30) - Not Null
    #[sqlx(rename = "START_DATE")]
    pub start_date: DateTime<Utc>, // DATE - Not Null
    #[sqlx(rename = "END_DATE")]
    pub end_date: Option<DateTime<Utc>>, // DATE - Nullable
    #[sqlx(rename = "USER_GUID")]
    pub user_guid: String, // VARCHAR2(64) - Not Null
    #[sqlx(rename = "USERNAME")]
    pub username: Option<String>, // VARCHAR2(100) - Nullable
    #[sqlx(rename = "MULTITENANCY_USERNAME")]
    pub multitenancy_username: Option<String>, // VARCHAR2(255) - Nullable
    #[sqlx(rename = "PERSON_ID")]
    pub person_id: Option<i64>, // NUMBER(18) - Nullable
    #[sqlx(rename = "PARTY_ID")]
    pub party_id: Option<i64>, // NUMBER(18) - Nullable
    #[sqlx(rename = "OBJECT_VERSION_NUMBER")]
    pub object_version_number: i32, // NUMBER(9) - Not Null
    #[sqlx(rename = "CREATED_BY")]
    pub created_by: String, // VARCHAR2(64) - Not Null
    #[sqlx(rename = "CREATION_DATE")]
    pub creation_date: DateTime<Utc>, // TIMESTAMP - Not Null
    #[sqlx(rename = "LAST_UPDATED_BY")]
    pub last_updated_by: String, // VARCHAR2(64) - Not Null
    #[sqlx(rename = "LAST_UPDATE_DATE")]
    pub last_update_date: DateTime<Utc>, // TIMESTAMP - Not Null
    #[sqlx(rename = "LAST_UPDATE_LOGIN")]
    pub last_update_login: Option<String>, // VARCHAR2(32) - Nullable
    #[sqlx(rename = "HR_TERMINATED")]
    pub hr_terminated: Option<String>, // VARCHAR2(30) - Nullable
    #[sqlx(rename = "SUSPENDED")]
    pub suspended: Option<String>, // VARCHAR2(30) - Nullable
    #[sqlx(rename = "USER_DISTINGUISHED_NAME")]
    pub user_distinguished_name: Option<String>, // VARCHAR2(4000) - Nullable
    #[sqlx(rename = "USER_DATA_CHECKSUM")]
    pub user_data_checksum: Option<String>, // VARCHAR2(64) - Nullable
    #[sqlx(rename = "CREDENTIALS_EMAIL_SENT")]
    pub credentials_email_sent: String, // VARCHAR2(30) - Not Null
    #[sqlx(rename = "EXTERNAL_ID")]
    pub external_id: Option<String>, // VARCHAR2(64) - Nullable
}
