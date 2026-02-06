use tower_sessions_redis_store::fred::prelude::RedisPool;
use sqlx::MySqlPool;

#[derive(Clone)]
pub struct AppState {
    pub db: MySqlPool,
    pub redis: RedisPool,
}
