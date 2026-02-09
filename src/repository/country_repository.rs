use sqlx::MySqlPool;
use crate::models::entities::Country;

pub async fn get_countries(pool: &MySqlPool) -> Result<Vec<Country>, sqlx::Error> {
    let rows = sqlx::query_as::<_, Country>("SELECT id, name FROM countries ORDER BY name ASC")
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn get_country_by_id(
    pool: &MySqlPool,
    country_id: i32,
) -> Result<Option<Country>, sqlx::Error> {
    let row = sqlx::query_as::<_, Country>(
        "SELECT id, name FROM countries WHERE id = ?",
    )
    .bind(country_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn create_country(pool: &MySqlPool, name: &str) -> Result<i32, sqlx::Error> {
    let result = sqlx::query("INSERT INTO countries (name) VALUES (?)")
        .bind(name)
        .execute(pool)
        .await?;
    Ok(result.last_insert_id() as i32)
}

pub async fn update_country(
    pool: &MySqlPool,
    country_id: i32,
    name: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!("UPDATE countries SET name = ? WHERE id = ?", name, country_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_country(pool: &MySqlPool, country_id: i32) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM countries WHERE id = ?", country_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn count_states_by_country_id(
    pool: &MySqlPool,
    country_id: i32,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT COUNT(*) as count FROM states WHERE country_id = ?",
        country_id
    )
    .fetch_one(pool)
    .await?;
    Ok(result.count)
}

pub async fn count_users_by_country_id(
    pool: &MySqlPool,
    country_id: i32,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT COUNT(*) as count FROM users WHERE country_id = ?",
        country_id
    )
    .fetch_one(pool)
    .await?;
    Ok(result.count)
}
