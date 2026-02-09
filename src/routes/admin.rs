use axum::{Router, routing::{get, post}};

use crate::controllers::page_controller;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(page_controller::admin_index))
        .route("/dashboard", get(page_controller::admin_dashboard))
        .route("/logout", get(page_controller::admin_logout))
        .route(
            "/countries",
            get(page_controller::admin_countries_list)
                .post(page_controller::admin_country_create_submit),
        )
        .route("/countries/new", get(page_controller::admin_country_create_page))
        .route("/countries/:id", post(page_controller::admin_country_edit_submit))
        .route("/countries/:id/edit", get(page_controller::admin_country_edit_page))
        .route("/countries/:id/delete", post(page_controller::admin_country_delete))
        .route(
            "/states",
            get(page_controller::admin_states_list)
                .post(page_controller::admin_state_create_submit),
        )
        .route("/states/new", get(page_controller::admin_state_create_page))
        .route("/states/data", get(page_controller::admin_states_api))
        .route("/geo/states", get(page_controller::admin_states_api))
        .route("/states/:id", post(page_controller::admin_state_edit_submit))
        .route("/states/:id/edit", get(page_controller::admin_state_edit_page))
        .route("/states/:id/delete", post(page_controller::admin_state_delete))
        .route(
            "/users",
            get(page_controller::users_list).post(page_controller::user_create_submit),
        )
        .route("/users/print", get(page_controller::admin_users_pdf))
        .route("/users/new", get(page_controller::user_create_page))
        .route(
            "/users/:id",
            get(page_controller::user_detail).post(page_controller::user_edit_submit),
        )
        .route("/users/:id/edit", get(page_controller::user_edit_page))
        .route("/users/:id/delete", post(page_controller::user_delete))
}
