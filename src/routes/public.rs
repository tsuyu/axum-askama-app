use axum::{Router, routing::get};

use crate::controllers::page_controller;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(page_controller::index))
        .route(
            "/login",
            get(page_controller::login_page).post(page_controller::login_submit),
        )
        .route(
            "/register",
            get(page_controller::register_page).post(page_controller::register_submit),
        )
        .route(
            "/password",
            get(page_controller::update_password_page)
                .post(page_controller::update_password_submit),
        )
        .route("/logout", get(page_controller::logout))
}
