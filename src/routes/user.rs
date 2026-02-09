use axum::{Router, routing::get};

use crate::controllers::page_controller;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/dashboard", get(page_controller::user_dashboard))
        .route(
            "/password",
            get(page_controller::update_password_page)
                .post(page_controller::update_password_submit),
        )
}
