use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    #[serde(serialize_with = "crate::utils::serialize_datetime")]
    pub created_at: OffsetDateTime,
    pub address: Option<String>,
    pub country_id: Option<i32>,
    pub state_id: Option<i32>,
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
