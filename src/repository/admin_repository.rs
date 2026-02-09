use sqlx::MySqlPool;
use crate::models::entities::Admin;

pub async fn find_admin_by_username(
    pool: &MySqlPool,
    username: &str,
) -> Result<Option<Admin>, sqlx::Error> {
    let admin = sqlx::query_as::<_, Admin>(
        "SELECT id, username, email, password_hash, created_at FROM admins WHERE username = ?",
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    Ok(admin)
}

pub async fn find_admin_by_id(
    pool: &MySqlPool,
    admin_id: i32,
) -> Result<Option<Admin>, sqlx::Error> {
    let admin = sqlx::query_as::<_, Admin>(
        "SELECT id, username, email, password_hash, created_at FROM admins WHERE id = ?",
    )
    .bind(admin_id)
    .fetch_optional(pool)
    .await?;

    Ok(admin)
}
