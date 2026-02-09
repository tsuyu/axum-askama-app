use serde::{Deserialize, Serialize};

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
