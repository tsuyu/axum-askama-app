use sqlx::MySqlPool;
use crate::models::entities::{User, PaginationParams};

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
