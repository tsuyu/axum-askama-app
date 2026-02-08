use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use crate::models::utils::{serialize_datetime, serialize_datetime_option};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    #[serde(serialize_with = "serialize_datetime")]
    pub created_at: OffsetDateTime,
    pub address: Option<String>,
    pub country_id: Option<i32>,
    pub state_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Admin {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    #[serde(serialize_with = "serialize_datetime_option")]
    pub created_at: Option<OffsetDateTime>,
}

// DataTables pagination parameters
#[derive(Debug)]
pub struct PaginationParams {
    pub offset: i64,
    pub limit: i64,
    pub search: Option<String>,
    pub order_column: String,
    pub order_direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Country {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct State {
    pub id: i32,
    pub country_id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StateWithCountry {
    pub id: i32,
    pub country_id: i32,
    pub name: String,
    pub country_name: String,
}
