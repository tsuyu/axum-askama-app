mod controllers;
mod models;
mod repository;
mod pool;
mod utils;
mod views;
mod routes;
mod state;

use std::net::SocketAddr;
use time::Duration;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_redis_store::{
    fred::interfaces::ClientLike,
    fred::prelude::{RedisConfig, RedisPool},
    RedisStore,
};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use std::env;
use crate::routes::app;
use crate::state::AppState;

#[tokio::main]
async fn main() {
    let env_name = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
    let log_dir = env::var("LOG_DIR").ok();

    // Set default log level based on environment
    let default_filter = if env_name == "development" {
        "debug,sqlx=info,tower_http=info"
    } else {
        "info,tower_http=info"
    };

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(default_filter));

    let registry = tracing_subscriber::registry().with(filter);
    let mut _log_guard: Option<tracing_appender::non_blocking::WorkerGuard> = None;

    if env_name == "production" || log_dir.is_some() {
        let dir = log_dir.unwrap_or_else(|| "logs".to_string());
        if let Err(err) = std::fs::create_dir_all(&dir) {
            eprintln!("Failed to create log dir {}: {}", dir, err);
        }
        let file_appender = tracing_appender::rolling::daily(dir, "app.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        _log_guard = Some(guard);

        registry
            .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
            .init();

    } else {
        registry
            .with(fmt::layer().pretty().with_thread_ids(true).with_target(true))
            .init();
    }

    tracing::info!("Application starting in {} mode", env_name);

    // Load environment variables
    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    // Create database connection pool
    let pool = pool::create_pool(&database_url)
        .await
        .expect("Failed to create database pool");

    tracing::info!("Database connection established");

    // Create Redis session store
    let redis_url =
        std::env::var("REDIS_URL").expect("REDIS_URL must be set in .env file");
    let redis_config = RedisConfig::from_url(redis_url.as_str()).expect("Invalid REDIS_URL");
    let redis_pool: RedisPool = RedisPool::new(redis_config, None, None, None, 6)
        .expect("Failed to create Redis pool");
    let _redis_conn = redis_pool.connect();
    let connect_result: Result<(), tower_sessions_redis_store::fred::error::RedisError> =
        redis_pool.wait_for_connect().await;
    connect_result.expect("Failed to connect to Redis");

    let session_store = RedisStore::new(redis_pool.clone());

    let session_timeout_secs: i64 = std::env::var("SESSION_TIMEOUT")
        .ok()
        .and_then(|val| val.parse::<i64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(60 * 60 * 24 * 7);

    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(session_timeout_secs)));

    // Build our application with routes
    let app_state = AppState {
        db: pool,
        redis: redis_pool,
    };
    let app = app(app_state, session_layer);

    // Run it
    let host = std::env::var("APP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = std::env::var("APP_PORT")
        .ok()
        .and_then(|val| val.parse().ok())
        .or_else(|| {
            std::env::var("PORT")
                .ok()
                .and_then(|val| val.parse().ok())
        })
        .unwrap_or(3001);

    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .expect("Invalid APP_HOST/APP_PORT");
    tracing::info!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
