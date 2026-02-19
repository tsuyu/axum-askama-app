use axum::{Router, routing::get};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tower_sessions::SessionManagerLayer;
use tower_sessions_redis_store::{fred::prelude::RedisPool, RedisStore};
use crate::state::AppState;

mod admin;
mod api;
mod public;
mod user;

pub fn app(
    state: AppState,
    session_layer: SessionManagerLayer<RedisStore<RedisPool>>,
) -> Router {
    let base_path = std::env::var("APP_BASE_PATH")
        .unwrap_or_default();
    // Normalize: strip trailing slash, keep leading slash (e.g. "/boilerplate")
    let base_path = base_path.trim_end_matches('/').to_string();

    let inner = Router::new()
        .merge(public::routes())
        .nest("/user", user::routes())
        .nest("/admin", admin::routes())
        .nest("/api", api::routes())
        .nest_service("/static", ServeDir::new("static"));

    // Register the home route directly at the correct level so it is always
    // reachable, regardless of how nest() handles the inner "/" route.
    let router: Router<AppState> = if base_path.is_empty() {
        inner
            .route("/", get(crate::controllers::page_controller::index))
            .fallback(crate::controllers::page_controller::handle_404)
    } else {
        let home = format!("{}/", base_path);
        Router::<AppState>::new()
            .route(&home, get(crate::controllers::page_controller::index))
            .nest(&base_path, inner)
            .fallback(crate::controllers::page_controller::handle_404)
    };

    router
        .layer(TraceLayer::new_for_http())
        .layer(session_layer)
        .with_state(state)
}
