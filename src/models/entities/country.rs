use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Country {
    pub id: i32,
    pub name: String,
}

// Form structs for controllers
#[derive(Debug, Deserialize, validator::Validate)]
pub struct CountryForm {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(length(min = 1))]
    pub csrf_token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CountryOption {
    pub id: i32,
    pub name: String,
}