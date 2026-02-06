mod controllers;
mod models;
mod views;
mod routes;
mod state;

use std::net::SocketAddr;
use time::Duration;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_redis_store::{fred::prelude::*, RedisStore};
use crate::models::db;
use crate::routes::app;
use crate::state::AppState;

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    // Create database connection pool
    let pool = db::create_pool(&database_url)
        .await
        .expect("Failed to create database pool");

    println!("Database connection established");

    // Create Redis session store
    let redis_url =
        std::env::var("REDIS_URL").expect("REDIS_URL must be set in .env file");
    let redis_config = RedisConfig::from_url(redis_url.as_str()).expect("Invalid REDIS_URL");
    let redis_pool = RedisPool::new(redis_config, None, None, None, 6)
        .expect("Failed to create Redis pool");
    let _redis_conn = redis_pool.connect();
    redis_pool
        .wait_for_connect()
        .await
        .expect("Failed to connect to Redis");

    let session_store = RedisStore::new(redis_pool.clone());

    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(Duration::days(7)));

    // Build our application with routes
    let app_state = AppState {
        db: pool,
        redis: redis_pool,
    };
    let app = app(app_state, session_layer);

    // Run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    println!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
