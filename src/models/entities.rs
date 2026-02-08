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

// DataTables request parameters
#[derive(Debug, Deserialize)]
pub struct DatatableParams {
    pub draw: u32,
    pub start: Option<i64>,
    pub length: Option<i64>,
    #[serde(rename = "search[value]")]
    pub search_value: Option<String>,
    #[serde(rename = "order[0][column]")]
    pub order_column: Option<i32>,
    #[serde(rename = "order[0][dir]")]
    pub order_dir: Option<String>,
}

// DataTables response format
#[derive(Debug, Serialize)]
pub struct DatatableResponse<T> {
    pub draw: u32,
    #[serde(rename = "recordsTotal")]
    pub records_total: i64,
    #[serde(rename = "recordsFiltered")]
    pub records_filtered: i64,
    pub data: Vec<T>,
}

// Form structs for controllers
#[derive(Debug, Deserialize, validator::Validate)]
pub struct CountryForm {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(length(min = 1))]
    pub csrf_token: String,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct StateForm {
    #[validate(range(min = 1))]
    pub country_id: i32,
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(length(min = 1))]
    pub csrf_token: String,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct CreateUserForm {
    #[validate(length(min = 1))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
    #[validate(length(min = 1))]
    pub address: String,
    #[validate(range(min = 1))]
    pub country_id: i32,
    #[validate(range(min = 1))]
    pub state_id: i32,
    #[validate(length(min = 1))]
    pub csrf_token: String,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct LoginForm {
    #[validate(length(min = 1))]
    pub username: String,
    #[validate(length(min = 6))]
    pub password: String,
    #[validate(length(min = 1))]
    pub csrf_token: String,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct RegisterForm {
    #[validate(length(min = 1))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
    #[validate(length(min = 1))]
    pub csrf_token: String,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct CsrfOnlyForm {
    #[validate(length(min = 1))]
    pub csrf_token: String,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct UpdateUserForm {
    #[validate(length(min = 1))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub address: String,
    #[validate(range(min = 1))]
    pub country_id: i32,
    #[validate(range(min = 1))]
    pub state_id: i32,
    #[serde(default)]
    pub new_password: String,
    #[validate(length(min = 1))]
    pub csrf_token: String,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct UpdatePasswordForm {
    #[validate(length(min = 6))]
    pub current_password: String,
    #[validate(length(min = 6))]
    pub new_password: String,
    #[validate(length(min = 6))]
    pub confirm_password: String,
    #[validate(length(min = 1))]
    pub csrf_token: String,
}

// DataTables request/response structs
#[derive(Debug, Deserialize)]
pub struct DataTablesRequest {
    pub draw: i32,
    pub start: i64,
    pub length: i64,
    #[serde(default)]
    pub search: DataTablesSearch,
    #[serde(default)]
    pub order: Vec<DataTablesOrder>,
}

#[derive(Debug, Deserialize, Default)]
pub struct DataTablesSearch {
    #[serde(default)]
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct DataTablesOrder {
    pub column: usize,
    pub dir: String,
}

#[derive(Debug, Deserialize)]
pub struct StatesQuery {
    pub country_id: i32,
}

#[derive(Debug, Serialize)]
pub struct DataTablesResponseLegacy {
    pub draw: i32,
    #[serde(rename = "recordsTotal")]
    pub records_total: i64,
    #[serde(rename = "recordsFiltered")]
    pub records_filtered: i64,
    pub data: Vec<UserRow>,
}

#[derive(Debug, Serialize)]
pub struct UserRow {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct PdfExportParams {
    pub search: Option<String>,
    pub order_column: Option<String>,
    pub order_direction: Option<String>,
}

// View data structures
#[derive(Debug, Serialize, Clone)]
pub struct UserView {
    pub id: u32,
    pub name: String,
    pub email: String,
    pub address: Option<String>,
    pub country: Option<String>,
    pub state: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CountryOption {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StateOption {
    pub id: i32,
    pub country_id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct AdminStateRow {
    pub id: i32,
    pub country_id: i32,
    pub country_name: String,
    pub name: String,
}