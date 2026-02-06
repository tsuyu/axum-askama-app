use serde::{Deserialize, Serialize};
use validator::Validate;
use sqlx::{MySqlPool, mysql::MySqlPoolOptions};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: OffsetDateTime,
    pub address: Option<String>,
    pub country_id: Option<i32>,
    pub state_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Admin {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: Option<OffsetDateTime>,
}


// DataTables pagination parameters
#[derive(Debug)]
pub struct PaginationParams {
    pub offset: i64,
    pub limit: i64,
    pub search: Option<String>,
    pub order_column: String,
    pub order_direction: String,
}

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

pub async fn create_user(
    pool: &MySqlPool,
    username: &str,
    email: &str,
    password: &str,
    address: Option<&str>,
    country_id: Option<i32>,
    state_id: Option<i32>,
) -> Result<i32, sqlx::Error> {
    let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| sqlx::Error::Protocol(format!("Password hashing failed: {}", e)))?;

    let result = sqlx::query(
        "INSERT INTO users (username, email, password_hash, address, country_id, state_id) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(username)
    .bind(email)
    .bind(password_hash)
    .bind(address)
    .bind(country_id)
    .bind(state_id)
    .execute(pool)
    .await?;

    Ok(result.last_insert_id() as i32)
}

pub async fn find_user_by_username(
    pool: &MySqlPool,
    username: &str,
) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, created_at, address, country_id, state_id FROM users WHERE username = ?",
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn find_user_by_id(
    pool: &MySqlPool,
    user_id: i32,
) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, created_at, address, country_id, state_id FROM users WHERE id = ?",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

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

pub async fn verify_password_hash(password_hash: &str, password: &str) -> bool {
    bcrypt::verify(password, password_hash).unwrap_or(false)
}

pub async fn update_password(
    pool: &MySqlPool,
    user_id: i32,
    new_password: &str,
) -> Result<(), sqlx::Error> {
    let password_hash = bcrypt::hash(new_password, bcrypt::DEFAULT_COST)
        .map_err(|e| sqlx::Error::Protocol(format!("Password hashing failed: {}", e)))?;

    sqlx::query!(
        "UPDATE users SET password_hash = ? WHERE id = ?",
        password_hash,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_user(
    pool: &MySqlPool,
    user_id: i32,
    username: &str,
    email: &str,
    address: Option<&str>,
    country_id: Option<i32>,
    state_id: Option<i32>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE users SET username = ?, email = ?, address = ?, country_id = ?, state_id = ? WHERE id = ?",
        username,
        email,
        address,
        country_id,
        state_id,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_user(pool: &MySqlPool, user_id: i32) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM users WHERE id = ?", user_id)
        .execute(pool)
        .await?;
    Ok(())
}

// Get total count of users
pub async fn get_users_count(pool: &MySqlPool) -> Result<i64, sqlx::Error> {
    let result = sqlx::query!("SELECT COUNT(*) as count FROM users")
        .fetch_one(pool)
        .await?;

    Ok(result.count)
}

// Get users with pagination and sorting
pub async fn get_users_paginated(
    pool: &MySqlPool,
    params: &PaginationParams,
) -> Result<Vec<User>, sqlx::Error> {
    // Validate and sanitize order column to prevent SQL injection
    let order_column = match params.order_column.as_str() {
        "id" => "id",
        "username" => "username",
        "email" => "email",
        "created_at" => "created_at",
        _ => "id", // default
    };

    let order_direction = match params.order_direction.as_str() {
        "asc" | "ASC" => "ASC",
        "desc" | "DESC" => "DESC",
        _ => "DESC", // default
    };

    let query_str = if let Some(search) = &params.search {
        if !search.is_empty() {
            format!(
                "SELECT id, username, email, password_hash, created_at, address, country_id, state_id 
                 FROM users 
                 WHERE username LIKE ? OR email LIKE ?
                 ORDER BY {} {} 
                 LIMIT ? OFFSET ?",
                order_column, order_direction
            )
        } else {
            format!(
                "SELECT id, username, email, password_hash, created_at, address, country_id, state_id 
                 FROM users 
                 ORDER BY {} {} 
                 LIMIT ? OFFSET ?",
                order_column, order_direction
            )
        }
    } else {
        format!(
            "SELECT id, username, email, password_hash, created_at, address, country_id, state_id 
             FROM users 
             ORDER BY {} {} 
             LIMIT ? OFFSET ?",
            order_column, order_direction
        )
    };

    let users = if let Some(search) = &params.search {
        if !search.is_empty() {
            let search_pattern = format!("%{}%", search);
            sqlx::query_as::<_, User>(&query_str)
                .bind(&search_pattern)
                .bind(&search_pattern)
                .bind(params.limit)
                .bind(params.offset)
                .fetch_all(pool)
                .await?
        } else {
            sqlx::query_as::<_, User>(&query_str)
                .bind(params.limit)
                .bind(params.offset)
                .fetch_all(pool)
                .await?
        }
    } else {
        sqlx::query_as::<_, User>(&query_str)
            .bind(params.limit)
            .bind(params.offset)
            .fetch_all(pool)
            .await?
    };

    Ok(users)
}

pub async fn get_users_for_export(
    pool: &MySqlPool,
    search: &Option<String>,
    order_column: &str,
    order_direction: &str,
) -> Result<Vec<User>, sqlx::Error> {
    // Validate and sanitize order column to prevent SQL injection
    let order_column = match order_column {
        "id" => "id",
        "username" => "username",
        "email" => "email",
        "created_at" => "created_at",
        _ => "id",
    };

    let order_direction = match order_direction {
        "asc" | "ASC" => "ASC",
        "desc" | "DESC" => "DESC",
        _ => "DESC",
    };

    let query_str = if let Some(search) = search {
        if !search.is_empty() {
            format!(
                "SELECT id, username, email, password_hash, created_at, address, country_id, state_id 
                 FROM users 
                 WHERE username LIKE ? OR email LIKE ?
                 ORDER BY {} {}",
                order_column, order_direction
            )
        } else {
            format!(
                "SELECT id, username, email, password_hash, created_at, address, country_id, state_id 
                 FROM users 
                 ORDER BY {} {}",
                order_column, order_direction
            )
        }
    } else {
        format!(
            "SELECT id, username, email, password_hash, created_at, address, country_id, state_id 
             FROM users 
             ORDER BY {} {}",
            order_column, order_direction
        )
    };

    let users = if let Some(search) = search {
        if !search.is_empty() {
            let search_pattern = format!("%{}%", search);
            sqlx::query_as::<_, User>(&query_str)
                .bind(&search_pattern)
                .bind(&search_pattern)
                .fetch_all(pool)
                .await?
        } else {
            sqlx::query_as::<_, User>(&query_str)
                .fetch_all(pool)
                .await?
        }
    } else {
        sqlx::query_as::<_, User>(&query_str)
            .fetch_all(pool)
            .await?
    };

    Ok(users)
}

// Get count of filtered users (for search)
pub async fn get_filtered_users_count(
    pool: &MySqlPool,
    search: &Option<String>,
) -> Result<i64, sqlx::Error> {
    let count = if let Some(search_term) = search {
        if !search_term.is_empty() {
            let search_pattern = format!("%{}%", search_term);
            let result = sqlx::query!(
                "SELECT COUNT(*) as count FROM users WHERE username LIKE ? OR email LIKE ?",
                search_pattern,
                search_pattern
            )
            .fetch_one(pool)
            .await?;
            result.count
        } else {
            get_users_count(pool).await?
        }
    } else {
        get_users_count(pool).await?
    };

    Ok(count)
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Country {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct State {
    pub id: i32,
    pub country_id: i32,
    pub name: String,
}

pub async fn get_countries(pool: &MySqlPool) -> Result<Vec<Country>, sqlx::Error> {
    let rows = sqlx::query_as::<_, Country>("SELECT id, name FROM countries ORDER BY name ASC")
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

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
