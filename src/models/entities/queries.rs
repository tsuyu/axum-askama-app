use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PdfExportParams {
    pub search: Option<String>,
    pub order_column: Option<String>,
    pub order_direction: Option<String>,
}
