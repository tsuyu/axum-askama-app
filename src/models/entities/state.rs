use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct State {
    pub id: i32,
    pub country_id: i32,
    pub name: String,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StateOption {
    pub id: i32,
    pub country_id: i32,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct StatesQuery {
    pub country_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StateWithCountry {
    pub id: i32,
    pub country_id: i32,
    pub name: String,
    pub country_name: String,
}


