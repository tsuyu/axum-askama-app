use axum::Router;
use crate::state::AppState;

mod v1;
mod v2;

pub fn routes() -> Router<AppState> {
    Router::new()
        .nest("/v1", v1::routes())
        .nest("/v2", v2::routes())
}
