use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use time::format_description::well_known::Rfc3339;

use crate::models;
use crate::repository;
use crate::state::AppState;

use super::shared::{DataTablesRequest, DataTablesResponse, UserRow};

pub async fn users_datatable_api(
    State(state): State<AppState>,
    Query(request): Query<DataTablesRequest>,
) -> impl IntoResponse {
    let search = request.search.value.trim().to_string();
    let search_opt = if search.is_empty() { None } else { Some(search) };

    let order = request.order.get(0);
    let order_column = match order.map(|o| o.column).unwrap_or(0) {
        0 => "id",
        1 => "username",
        2 => "email",
        3 => "created_at",
        _ => "id",
    };
    let order_direction = order
        .map(|o| o.dir.as_str())
        .unwrap_or("desc")
        .to_string();

    let params = models::PaginationParams {
        offset: request.start,
        limit: request.length,
        search: search_opt.clone(),
        order_column: order_column.to_string(),
        order_direction,
    };

    let total = match repository::get_users_count(&state.db).await {
        Ok(total) => total,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let filtered = match repository::get_filtered_users_count(&state.db, &search_opt).await {
        Ok(count) => count,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let users = match repository::get_users_paginated(&state.db, &params).await {
        Ok(rows) => rows,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let data: Vec<UserRow> = users
        .into_iter()
        .map(|u| UserRow {
            id: u.id,
            username: u.username,
            email: u.email,
            created_at: u.created_at.format(&Rfc3339).unwrap_or_default(),
        })
        .collect();

    Json(DataTablesResponse {
        draw: request.draw,
        records_total: total,
        records_filtered: filtered,
        data,
    })
    .into_response()
}
