use axum::http::StatusCode;
use rand::{Rng, distributions::Alphanumeric};
use serde::{Deserialize, Serialize};
use serde_json;
use tower_sessions::Session;
use tower_sessions_redis_store::fred::prelude::KeysInterface;
use tower_sessions_redis_store::fred::types::Expiration;
use validator::Validate;

use crate::models;
use crate::state::AppState;
use crate::views::templates::{CountryOption, StateOption};

// Re-export form and request/response structs from entities for convenience
pub(crate) use crate::models::{
    CountryForm, StateForm, CreateUserForm, LoginForm, RegisterForm, CsrfOnlyForm,
    UpdateUserForm, UpdatePasswordForm, DataTablesRequest, DataTablesSearch, DataTablesOrder,
    StatesQuery, DataTablesResponseLegacy, UserRow, PdfExportParams,
};

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

pub(crate) fn map_country_options(countries: Vec<models::Country>) -> Vec<CountryOption> {
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

    let countries = models::get_countries(&state.db)
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

    let states = models::get_states_by_country(&state.db, country_id)
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
    if let Ok(countries) = models::get_countries(&state.db).await {
        for country in countries {
            let key = format!("geo:states:{}", country.id);
            let _: Result<(), _> = state.redis.del(key).await;
        }
    }
}

