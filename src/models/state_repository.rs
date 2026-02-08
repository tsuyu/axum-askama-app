use sqlx::MySqlPool;
use crate::models::entities::{State, StateWithCountry};

pub async fn get_states_by_country(
    pool: &MySqlPool,
    country_id: i32,
) -> Result<Vec<State>, sqlx::Error> {
    let rows = sqlx::query_as::<_, State>(
        "SELECT id, country_id, name FROM states WHERE country_id = ? ORDER BY name ASC",
    )
    .bind(country_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn get_state_by_id(
    pool: &MySqlPool,
    state_id: i32,
) -> Result<Option<State>, sqlx::Error> {
    let row = sqlx::query_as::<_, State>(
        "SELECT id, country_id, name FROM states WHERE id = ?",
    )
    .bind(state_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn get_states_with_countries(
    pool: &MySqlPool,
) -> Result<Vec<StateWithCountry>, sqlx::Error> {
    let rows = sqlx::query_as::<_, StateWithCountry>(
        "SELECT s.id, s.country_id, s.name, c.name as country_name
         FROM states s
         JOIN countries c ON c.id = s.country_id
         ORDER BY c.name ASC, s.name ASC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn create_state(
    pool: &MySqlPool,
    country_id: i32,
    name: &str,
) -> Result<i32, sqlx::Error> {
    let result = sqlx::query("INSERT INTO states (country_id, name) VALUES (?, ?)")
        .bind(country_id)
        .bind(name)
        .execute(pool)
        .await?;
    Ok(result.last_insert_id() as i32)
}

pub async fn update_state(
    pool: &MySqlPool,
    state_id: i32,
    country_id: i32,
    name: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE states SET country_id = ?, name = ? WHERE id = ?",
        country_id,
        name,
        state_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_state(pool: &MySqlPool, state_id: i32) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM states WHERE id = ?", state_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn count_users_by_state_id(
    pool: &MySqlPool,
    state_id: i32,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT COUNT(*) as count FROM users WHERE state_id = ?",
        state_id
    )
    .fetch_one(pool)
    .await?;
    Ok(result.count)
}
