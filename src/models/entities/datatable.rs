use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize)]
pub struct DataTablesResponseLegacy {
    pub draw: i32,
    pub records_total: i64,
    pub records_filtered: i64,
    pub data: Vec<UserRow>,
}

#[derive(Debug)]
pub struct PaginationParams {
    pub offset: i64,
    pub limit: i64,
    pub search: Option<String>,
    pub order_column: String,
    pub order_direction: String,
}


#[derive(Debug, Serialize)]
pub struct UserRow {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: String,
}
