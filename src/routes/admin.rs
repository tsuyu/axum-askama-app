use axum::{Router, routing::{get, post}};

use crate::controllers::page_controller;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(page_controller::admin_index))
        .route(
            "/login",
            get(page_controller::admin_login_page).post(page_controller::admin_login_submit),
        )
        .route("/logout", get(page_controller::admin_logout))
        .route("/geo/states", get(page_controller::admin_states_api))
        .route("/users/print", get(page_controller::admin_users_pdf))
        .route("/users", get(page_controller::users_list).post(page_controller::user_create_submit))
        .route("/users/new", get(page_controller::user_create_page))
        .route(
            "/users/:id",
            get(page_controller::user_detail).post(page_controller::user_edit_submit),
        )
        .route("/users/:id/edit", get(page_controller::user_edit_page))
        .route("/users/:id/delete", post(page_controller::user_delete))
}
