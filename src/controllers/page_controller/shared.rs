use axum::http::StatusCode;
use rand::{Rng, distributions::Alphanumeric};
use serde::{Deserialize, Serialize};
use serde_json;
use tower_sessions::Session;
use tower_sessions_redis_store::fred::prelude::KeysInterface;
use tower_sessions_redis_store::fred::types::Expiration;
use validator::Validate;

use crate::models::db;
use crate::state::AppState;
use crate::views::templates::{CountryOption, StateOption};

const CSRF_KEY: &str = "csrf_token";
const CACHE_TTL_SECONDS: i64 = 300;

fn generate_csrf_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

pub(crate) async fn ensure_csrf_token(session: &Session) -> String {
    if let Ok(Some(token)) = session.get::<String>(CSRF_KEY).await {
        token
    } else {
        let token = generate_csrf_token();
        let _ = session.insert(CSRF_KEY, token.clone()).await;
        token
    }
}

pub(crate) async fn validate_csrf(session: &Session, token: &str) -> bool {
    match session.get::<String>(CSRF_KEY).await {
        Ok(Some(stored)) => stored == token,
        _ => false,
    }
}

pub(crate) fn map_country_options(countries: Vec<db::Country>) -> Vec<CountryOption> {
    countries
        .into_iter()
        .map(|c| CountryOption { id: c.id, name: c.name })
        .collect()
}

pub(crate) async fn get_countries_cached(state: &AppState) -> Result<Vec<CountryOption>, StatusCode> {
    let key = "geo:countries";
    let cached: Option<String> = match state.redis.get(key).await {
        Ok(value) => value,
        Err(_) => None,
    };
    if let Some(json) = cached {
        if let Ok(cached) = serde_json::from_str::<Vec<CountryOption>>(&json) {
            return Ok(cached);
        }
    }

    let countries = db::get_countries(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let options = map_country_options(countries);

    if let Ok(json) = serde_json::to_string(&options) {
        let _ = state
            .redis
            .set::<(), _, _>(key, json, Some(Expiration::EX(CACHE_TTL_SECONDS)), None, false)
            .await;
    }

    Ok(options)
}

pub(crate) async fn get_states_cached(
    state: &AppState,
    country_id: i32,
) -> Result<Vec<StateOption>, StatusCode> {
    let key = format!("geo:states:{}", country_id);
    let cached: Option<String> = match state.redis.get(key.clone()).await {
        Ok(value) => value,
        Err(_) => None,
    };
    if let Some(json) = cached {
        if let Ok(cached) = serde_json::from_str::<Vec<StateOption>>(&json) {
            return Ok(cached);
        }
    }

    let states = db::get_states_by_country(&state.db, country_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let options: Vec<StateOption> = states
        .into_iter()
        .map(|s| StateOption { id: s.id, country_id: s.country_id, name: s.name })
        .collect();

    if let Ok(json) = serde_json::to_string(&options) {
        let _ = state
            .redis
            .set::<(), _, _>(key, json, Some(Expiration::EX(CACHE_TTL_SECONDS)), None, false)
            .await;
    }

    Ok(options)
}

pub(crate) async fn invalidate_geo_cache(state: &AppState) {
    let _: Result<(), _> = state.redis.del("geo:countries").await;
    if let Ok(countries) = db::get_countries(&state.db).await {
        for country in countries {
            let key = format!("geo:states:{}", country.id);
            let _: Result<(), _> = state.redis.del(key).await;
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub(crate) struct CountryForm {
    #[validate(length(min = 1))]
    pub(crate) name: String,
    #[validate(length(min = 1))]
    pub(crate) csrf_token: String,
}

#[derive(Debug, Deserialize, Validate)]
pub(crate) struct StateForm {
    #[validate(range(min = 1))]
    pub(crate) country_id: i32,
    #[validate(length(min = 1))]
    pub(crate) name: String,
    #[validate(length(min = 1))]
    pub(crate) csrf_token: String,
}

#[derive(Debug, Deserialize, Validate)]
pub(crate) struct CreateUserForm {
    #[validate(length(min = 1))]
    pub(crate) username: String,
    #[validate(email)]
    pub(crate) email: String,
    #[validate(length(min = 6))]
    pub(crate) password: String,
    #[validate(length(min = 1))]
    pub(crate) address: String,
    #[validate(range(min = 1))]
    pub(crate) country_id: i32,
    #[validate(range(min = 1))]
    pub(crate) state_id: i32,
    #[validate(length(min = 1))]
    pub(crate) csrf_token: String,
}

#[derive(Debug, Deserialize, Validate)]
pub(crate) struct LoginForm {
    #[validate(length(min = 1))]
    pub(crate) username: String,
    #[validate(length(min = 6))]
    pub(crate) password: String,
    #[validate(length(min = 1))]
    pub(crate) csrf_token: String,
}

#[derive(Debug, Deserialize, Validate)]
pub(crate) struct RegisterForm {
    #[validate(length(min = 1))]
    pub(crate) username: String,
    #[validate(email)]
    pub(crate) email: String,
    #[validate(length(min = 6))]
    pub(crate) password: String,
    #[validate(length(min = 1))]
    pub(crate) csrf_token: String,
}

#[derive(Debug, Deserialize, Validate)]
pub(crate) struct CsrfOnlyForm {
    #[validate(length(min = 1))]
    pub(crate) csrf_token: String,
}

#[derive(Debug, Deserialize, Validate)]
pub(crate) struct UpdateUserForm {
    #[validate(length(min = 1))]
    pub(crate) username: String,
    #[validate(email)]
    pub(crate) email: String,
    #[validate(length(min = 1))]
    pub(crate) address: String,
    #[validate(range(min = 1))]
    pub(crate) country_id: i32,
    #[validate(range(min = 1))]
    pub(crate) state_id: i32,
    #[serde(default)]
    pub(crate) new_password: String,
    #[validate(length(min = 1))]
    pub(crate) csrf_token: String,
}

#[derive(Debug, Deserialize, Validate)]
pub(crate) struct UpdatePasswordForm {
    #[validate(length(min = 6))]
    pub(crate) current_password: String,
    #[validate(length(min = 6))]
    pub(crate) new_password: String,
    #[validate(length(min = 6))]
    pub(crate) confirm_password: String,
    #[validate(length(min = 1))]
    pub(crate) csrf_token: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DataTablesRequest {
    pub(crate) draw: i32,
    pub(crate) start: i64,
    pub(crate) length: i64,
    #[serde(default)]
    pub(crate) search: DataTablesSearch,
    #[serde(default)]
    pub(crate) order: Vec<DataTablesOrder>,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct DataTablesSearch {
    #[serde(default)]
    pub(crate) value: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DataTablesOrder {
    pub(crate) column: usize,
    pub(crate) dir: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct StatesQuery {
    pub(crate) country_id: i32,
}

#[derive(Debug, Serialize)]
pub(crate) struct DataTablesResponse {
    pub(crate) draw: i32,
    #[serde(rename = "recordsTotal")]
    pub(crate) records_total: i64,
    #[serde(rename = "recordsFiltered")]
    pub(crate) records_filtered: i64,
    pub(crate) data: Vec<UserRow>,
}

#[derive(Debug, Serialize)]
pub(crate) struct UserRow {
    pub(crate) id: i32,
    pub(crate) username: String,
    pub(crate) email: String,
    pub(crate) created_at: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PdfExportParams {
    pub(crate) search: Option<String>,
    pub(crate) order_column: Option<String>,
    pub(crate) order_direction: Option<String>,
}
