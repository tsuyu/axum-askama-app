use axum::{Router, routing::get};

use crate::controllers::page_controller;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/login",
            get(page_controller::login_page).post(page_controller::login_submit),
        )
        .route(
            "/register",
            get(page_controller::register_page).post(page_controller::register_submit),
        )
        .route("/logout", get(page_controller::logout))
        .route(
            "/loginadmin",
            get(page_controller::admin_login_page).post(page_controller::admin_login_submit),
        )
}
