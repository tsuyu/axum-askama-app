use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct StatesQuery {
    pub country_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct PdfExportParams {
    pub search: Option<String>,
    pub order_column: Option<String>,
    pub order_direction: Option<String>,
}
