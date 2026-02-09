use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Admin {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    #[serde(serialize_with = "crate::utils::serialize_datetime_option")]
    pub created_at: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, Clone)]
pub struct AdminStateRow {
    pub id: i32,
    pub country_id: i32,
    pub country_name: String,
    pub name: String,
}



