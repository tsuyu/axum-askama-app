use sqlx::{MySqlPool, mysql::MySqlPoolOptions};

pub async fn create_pool(database_url: &str) -> Result<MySqlPool, sqlx::Error> {
    use std::time::Duration;

    MySqlPoolOptions::new()
        .max_connections(20)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(900))
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                // Set timezone to Malaysia (UTC+8) for every new connection
                sqlx::query("SET time_zone = '+08:00'")
                    .execute(&mut *conn)
                    .await?;
                Ok(())
            })
        })
        .connect(database_url)
        .await
}
