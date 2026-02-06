use axum::{
    Form,
    extract::{Extension, Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Json, Redirect},
};
use printpdf::{BuiltinFont, Mm, PdfDocument};
use rand::{Rng, distributions::Alphanumeric};
use std::io::BufWriter;
use serde::{Deserialize, Serialize};
use serde_json;
use time::format_description::well_known::Rfc3339;
use tower_sessions::Session;
use validator::Validate;
use tower_sessions_redis_store::fred::types::Expiration;
use tower_sessions_redis_store::fred::prelude::KeysInterface;

use crate::controllers::auth_controller::{
    AdminUser, AuthUser, OptionalAdminUser, OptionalAuthUser,
};
use crate::models::db::{self, PaginationParams};
use crate::state::AppState;
use crate::views::templates::{
    AdminCreateUserTemplate, AdminEditUserTemplate, AdminErrorTemplate, AdminLoginTemplate,
    AdminUserDetailTemplate, AdminUsersListTemplate, AdminCountriesListTemplate,
    AdminCountryFormTemplate, AdminStatesListTemplate, AdminStateFormTemplate, AdminStateRow,
    ErrorTemplate, IndexTemplate, LoginTemplate, RegisterTemplate, UpdatePasswordTemplate, User,
    CountryOption, StateOption,
};

const CSRF_KEY: &str = "csrf_token";

fn generate_csrf_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

async fn ensure_csrf_token(session: &Session) -> String {
    if let Ok(Some(token)) = session.get::<String>(CSRF_KEY).await {
        token
    } else {
        let token = generate_csrf_token();
        let _ = session.insert(CSRF_KEY, token.clone()).await;
        token
    }
}

async fn validate_csrf(session: &Session, token: &str) -> bool {
    match session.get::<String>(CSRF_KEY).await {
        Ok(Some(stored)) => stored == token,
        _ => false,
    }
}

const CACHE_TTL_SECONDS: i64 = 300;

fn map_country_options(countries: Vec<db::Country>) -> Vec<CountryOption> {
    countries
        .into_iter()
        .map(|c| CountryOption { id: c.id, name: c.name })
        .collect()
}

async fn get_countries_cached(state: &AppState) -> Result<Vec<CountryOption>, StatusCode> {
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

async fn get_states_cached(state: &AppState, country_id: i32) -> Result<Vec<StateOption>, StatusCode> {
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

async fn invalidate_geo_cache(state: &AppState) {
    let _: Result<(), _> = state.redis.del("geo:countries").await;
    if let Ok(countries) = db::get_countries(&state.db).await {
        for country in countries {
            let key = format!("geo:states:{}", country.id);
            let _: Result<(), _> = state.redis.del(key).await;
        }
    }
}

// Index handler
pub async fn index(
    OptionalAuthUser(user): OptionalAuthUser,
    Extension(session): Extension<Session>,
) -> impl IntoResponse {
    let flash_success = match session.get::<String>("flash_success").await {
        Ok(Some(msg)) => {
            let _ = session.remove::<String>("flash_success").await;
            Some(msg)
        }
        _ => None,
    };

    let template = IndexTemplate {
        title: "Welcome".to_string(),
        message: "Hello from Axum + Askama!".to_string(),
        user: user.map(|u| u.username),
        flash_success,
    };

    template
}

// Users list handler - requires authentication
pub async fn users_list(admin_user: AdminUser) -> impl IntoResponse {
    let template = AdminUsersListTemplate {
        page_title: "All Users".to_string(),
        current_admin: Some(admin_user.username),
    };

    template
}

#[derive(Debug, Deserialize, Validate)]
pub struct CountryForm {
    #[validate(length(min = 1))]
    name: String,
    #[validate(length(min = 1))]
    csrf_token: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct StateForm {
    #[validate(range(min = 1))]
    country_id: i32,
    #[validate(length(min = 1))]
    name: String,
    #[validate(length(min = 1))]
    csrf_token: String,
}

// Countries list (admin)
pub async fn admin_countries_list(
    admin_user: AdminUser,
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
) -> impl IntoResponse {
    let countries = match db::get_countries(&state.db).await {
        Ok(rows) => map_country_options(rows),
        Err(_) => {
            let template = AdminErrorTemplate {
                error_code: 500,
                error_message: "Failed to load countries.".to_string(),
                current_admin: Some(admin_user.username),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, template).into_response();
        }
    };

    AdminCountriesListTemplate {
        page_title: "Countries".to_string(),
        current_admin: Some(admin_user.username),
        csrf_token: ensure_csrf_token(&session).await,
        countries,
    }
    .into_response()
}

// Country create page (GET)
pub async fn admin_country_create_page(
    admin_user: AdminUser,
    Extension(session): Extension<Session>,
) -> impl IntoResponse {
    AdminCountryFormTemplate {
        form_title: "Create Country".to_string(),
        form_action: "/admin/countries".to_string(),
        submit_label: "Create Country".to_string(),
        country_id: None,
        name: None,
        error: None,
        success: None,
        current_admin: Some(admin_user.username),
        csrf_token: ensure_csrf_token(&session).await,
    }
    .into_response()
}

// Country create submission (POST)
pub async fn admin_country_create_submit(
    admin_user: AdminUser,
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Form(form): Form<CountryForm>,
) -> impl IntoResponse {
    let name = form.name.clone();
    if !validate_csrf(&session, &form.csrf_token).await {
        return AdminCountryFormTemplate {
            form_title: "Create Country".to_string(),
            form_action: "/admin/countries".to_string(),
            submit_label: "Create Country".to_string(),
            country_id: None,
            name: Some(name.clone()),
            error: Some("Invalid CSRF token".to_string()),
            success: None,
            current_admin: Some(admin_user.username),
            csrf_token: ensure_csrf_token(&session).await,
        }
        .into_response();
    }

    if form.validate().is_err() {
        return AdminCountryFormTemplate {
            form_title: "Create Country".to_string(),
            form_action: "/admin/countries".to_string(),
            submit_label: "Create Country".to_string(),
            country_id: None,
            name: Some(name.clone()),
            error: Some("Invalid country name".to_string()),
            success: None,
            current_admin: Some(admin_user.username),
            csrf_token: ensure_csrf_token(&session).await,
        }
        .into_response();
    }

    if let Err(_) = db::create_country(&state.db, &form.name).await {
        return AdminCountryFormTemplate {
            form_title: "Create Country".to_string(),
            form_action: "/admin/countries".to_string(),
            submit_label: "Create Country".to_string(),
            country_id: None,
            name: Some(name.clone()),
            error: Some("Failed to create country".to_string()),
            success: None,
            current_admin: Some(admin_user.username),
            csrf_token: ensure_csrf_token(&session).await,
        }
        .into_response();
    }

    invalidate_geo_cache(&state).await;
    Redirect::to("/admin/countries").into_response()
}

// Country edit page (GET)
pub async fn admin_country_edit_page(
    admin_user: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Extension(session): Extension<Session>,
) -> impl IntoResponse {
    let country = match db::get_country_by_id(&state.db, id).await {
        Ok(Some(country)) => country,
        Ok(None) => {
            let template = AdminErrorTemplate {
                error_code: 404,
                error_message: "Country not found.".to_string(),
                current_admin: Some(admin_user.username),
            };
            return (StatusCode::NOT_FOUND, template).into_response();
        }
        Err(_) => {
            let template = AdminErrorTemplate {
                error_code: 500,
                error_message: "Failed to load country.".to_string(),
                current_admin: Some(admin_user.username),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, template).into_response();
        }
    };

    AdminCountryFormTemplate {
        form_title: "Edit Country".to_string(),
        form_action: format!("/admin/countries/{}", id),
        submit_label: "Save Changes".to_string(),
        country_id: Some(country.id),
        name: Some(country.name),
        error: None,
        success: None,
        current_admin: Some(admin_user.username),
        csrf_token: ensure_csrf_token(&session).await,
    }
    .into_response()
}

// Country edit submission (POST)
pub async fn admin_country_edit_submit(
    admin_user: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Extension(session): Extension<Session>,
    Form(form): Form<CountryForm>,
) -> impl IntoResponse {
    let name = form.name.clone();
    if !validate_csrf(&session, &form.csrf_token).await {
        return AdminCountryFormTemplate {
            form_title: "Edit Country".to_string(),
            form_action: format!("/admin/countries/{}", id),
            submit_label: "Save Changes".to_string(),
            country_id: Some(id),
            name: Some(name.clone()),
            error: Some("Invalid CSRF token".to_string()),
            success: None,
            current_admin: Some(admin_user.username),
            csrf_token: ensure_csrf_token(&session).await,
        }
        .into_response();
    }

    if form.validate().is_err() {
        return AdminCountryFormTemplate {
            form_title: "Edit Country".to_string(),
            form_action: format!("/admin/countries/{}", id),
            submit_label: "Save Changes".to_string(),
            country_id: Some(id),
            name: Some(name.clone()),
            error: Some("Invalid country name".to_string()),
            success: None,
            current_admin: Some(admin_user.username),
            csrf_token: ensure_csrf_token(&session).await,
        }
        .into_response();
    }

    if let Err(_) = db::update_country(&state.db, id, &form.name).await {
        return AdminCountryFormTemplate {
            form_title: "Edit Country".to_string(),
            form_action: format!("/admin/countries/{}", id),
            submit_label: "Save Changes".to_string(),
            country_id: Some(id),
            name: Some(name.clone()),
            error: Some("Failed to update country".to_string()),
            success: None,
            current_admin: Some(admin_user.username),
            csrf_token: ensure_csrf_token(&session).await,
        }
        .into_response();
    }

    invalidate_geo_cache(&state).await;
    Redirect::to("/admin/countries").into_response()
}

// Country delete (POST)
pub async fn admin_country_delete(
    admin_user: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Extension(session): Extension<Session>,
    Form(form): Form<CsrfOnlyForm>,
) -> impl IntoResponse {
    if !validate_csrf(&session, &form.csrf_token).await {
        let template = AdminErrorTemplate {
            error_code: 403,
            error_message: "Invalid CSRF token".to_string(),
            current_admin: Some(admin_user.username),
        };
        return (StatusCode::FORBIDDEN, template).into_response();
    }

    if let Ok(count) = db::count_states_by_country_id(&state.db, id).await {
        if count > 0 {
            let template = AdminErrorTemplate {
                error_code: 400,
                error_message: "Cannot delete country with existing states.".to_string(),
                current_admin: Some(admin_user.username),
            };
            return (StatusCode::BAD_REQUEST, template).into_response();
        }
    }

    if let Ok(count) = db::count_users_by_country_id(&state.db, id).await {
        if count > 0 {
            let template = AdminErrorTemplate {
                error_code: 400,
                error_message: "Cannot delete country assigned to users.".to_string(),
                current_admin: Some(admin_user.username),
            };
            return (StatusCode::BAD_REQUEST, template).into_response();
        }
    }

    if let Err(_) = db::delete_country(&state.db, id).await {
        let template = AdminErrorTemplate {
            error_code: 500,
            error_message: "Failed to delete country.".to_string(),
            current_admin: Some(admin_user.username),
        };
        return (StatusCode::INTERNAL_SERVER_ERROR, template).into_response();
    }

    invalidate_geo_cache(&state).await;
    Redirect::to("/admin/countries").into_response()
}

// States list (admin)
pub async fn admin_states_list(
    admin_user: AdminUser,
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
) -> impl IntoResponse {
    let states = match db::get_states_with_countries(&state.db).await {
        Ok(rows) => rows
            .into_iter()
            .map(|s| AdminStateRow {
                id: s.id,
                country_id: s.country_id,
                country_name: s.country_name,
                name: s.name,
            })
            .collect(),
        Err(_) => {
            let template = AdminErrorTemplate {
                error_code: 500,
                error_message: "Failed to load states.".to_string(),
                current_admin: Some(admin_user.username),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, template).into_response();
        }
    };

    AdminStatesListTemplate {
        page_title: "States".to_string(),
        current_admin: Some(admin_user.username),
        csrf_token: ensure_csrf_token(&session).await,
        states,
    }
    .into_response()
}

// State create page (GET)
pub async fn admin_state_create_page(
    admin_user: AdminUser,
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
) -> impl IntoResponse {
    let countries = match db::get_countries(&state.db).await {
        Ok(rows) => map_country_options(rows),
        Err(_) => Vec::new(),
    };

    AdminStateFormTemplate {
        form_title: "Create State".to_string(),
        form_action: "/admin/states".to_string(),
        submit_label: "Create State".to_string(),
        state_id: None,
        name: None,
        countries,
        selected_country_id: 0,
        error: None,
        success: None,
        current_admin: Some(admin_user.username),
        csrf_token: ensure_csrf_token(&session).await,
    }
    .into_response()
}

// State create submission (POST)
pub async fn admin_state_create_submit(
    admin_user: AdminUser,
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Form(form): Form<StateForm>,
) -> impl IntoResponse {
    let name = form.name.clone();
    if !validate_csrf(&session, &form.csrf_token).await {
        let countries = db::get_countries(&state.db)
            .await
            .map(map_country_options)
            .unwrap_or_default();
        return AdminStateFormTemplate {
            form_title: "Create State".to_string(),
            form_action: "/admin/states".to_string(),
            submit_label: "Create State".to_string(),
            state_id: None,
            name: Some(name.clone()),
            countries,
            selected_country_id: form.country_id,
            error: Some("Invalid CSRF token".to_string()),
            success: None,
            current_admin: Some(admin_user.username),
            csrf_token: ensure_csrf_token(&session).await,
        }
        .into_response();
    }

    if form.validate().is_err() {
        let countries = db::get_countries(&state.db)
            .await
            .map(map_country_options)
            .unwrap_or_default();
        return AdminStateFormTemplate {
            form_title: "Create State".to_string(),
            form_action: "/admin/states".to_string(),
            submit_label: "Create State".to_string(),
            state_id: None,
            name: Some(name.clone()),
            countries,
            selected_country_id: form.country_id,
            error: Some("Invalid state data".to_string()),
            success: None,
            current_admin: Some(admin_user.username),
            csrf_token: ensure_csrf_token(&session).await,
        }
        .into_response();
    }

    if let Err(_) = db::create_state(&state.db, form.country_id, &form.name).await {
        let countries = db::get_countries(&state.db)
            .await
            .map(map_country_options)
            .unwrap_or_default();
        return AdminStateFormTemplate {
            form_title: "Create State".to_string(),
            form_action: "/admin/states".to_string(),
            submit_label: "Create State".to_string(),
            state_id: None,
            name: Some(name.clone()),
            countries,
            selected_country_id: form.country_id,
            error: Some("Failed to create state".to_string()),
            success: None,
            current_admin: Some(admin_user.username),
            csrf_token: ensure_csrf_token(&session).await,
        }
        .into_response();
    }

    invalidate_geo_cache(&state).await;
    Redirect::to("/admin/states").into_response()
}

// State edit page (GET)
pub async fn admin_state_edit_page(
    admin_user: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Extension(session): Extension<Session>,
) -> impl IntoResponse {
    let state_row = match db::get_state_by_id(&state.db, id).await {
        Ok(Some(row)) => row,
        Ok(None) => {
            let template = AdminErrorTemplate {
                error_code: 404,
                error_message: "State not found.".to_string(),
                current_admin: Some(admin_user.username),
            };
            return (StatusCode::NOT_FOUND, template).into_response();
        }
        Err(_) => {
            let template = AdminErrorTemplate {
                error_code: 500,
                error_message: "Failed to load state.".to_string(),
                current_admin: Some(admin_user.username),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, template).into_response();
        }
    };

    let countries = db::get_countries(&state.db)
        .await
        .map(map_country_options)
        .unwrap_or_default();

    AdminStateFormTemplate {
        form_title: "Edit State".to_string(),
        form_action: format!("/admin/states/{}", id),
        submit_label: "Save Changes".to_string(),
        state_id: Some(state_row.id),
        name: Some(state_row.name),
        countries,
        selected_country_id: state_row.country_id,
        error: None,
        success: None,
        current_admin: Some(admin_user.username),
        csrf_token: ensure_csrf_token(&session).await,
    }
    .into_response()
}

// State edit submission (POST)
pub async fn admin_state_edit_submit(
    admin_user: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Extension(session): Extension<Session>,
    Form(form): Form<StateForm>,
) -> impl IntoResponse {
    let name = form.name.clone();
    if !validate_csrf(&session, &form.csrf_token).await {
        let countries = db::get_countries(&state.db)
            .await
            .map(map_country_options)
            .unwrap_or_default();
        return AdminStateFormTemplate {
            form_title: "Edit State".to_string(),
            form_action: format!("/admin/states/{}", id),
            submit_label: "Save Changes".to_string(),
            state_id: Some(id),
            name: Some(name.clone()),
            countries,
            selected_country_id: form.country_id,
            error: Some("Invalid CSRF token".to_string()),
            success: None,
            current_admin: Some(admin_user.username),
            csrf_token: ensure_csrf_token(&session).await,
        }
        .into_response();
    }

    if form.validate().is_err() {
        let countries = db::get_countries(&state.db)
            .await
            .map(map_country_options)
            .unwrap_or_default();
        return AdminStateFormTemplate {
            form_title: "Edit State".to_string(),
            form_action: format!("/admin/states/{}", id),
            submit_label: "Save Changes".to_string(),
            state_id: Some(id),
            name: Some(name.clone()),
            countries,
            selected_country_id: form.country_id,
            error: Some("Invalid state data".to_string()),
            success: None,
            current_admin: Some(admin_user.username),
            csrf_token: ensure_csrf_token(&session).await,
        }
        .into_response();
    }

    if let Err(_) = db::update_state(&state.db, id, form.country_id, &form.name).await {
        let countries = db::get_countries(&state.db)
            .await
            .map(map_country_options)
            .unwrap_or_default();
        return AdminStateFormTemplate {
            form_title: "Edit State".to_string(),
            form_action: format!("/admin/states/{}", id),
            submit_label: "Save Changes".to_string(),
            state_id: Some(id),
            name: Some(name.clone()),
            countries,
            selected_country_id: form.country_id,
            error: Some("Failed to update state".to_string()),
            success: None,
            current_admin: Some(admin_user.username),
            csrf_token: ensure_csrf_token(&session).await,
        }
        .into_response();
    }

    invalidate_geo_cache(&state).await;
    Redirect::to("/admin/states").into_response()
}

// State delete (POST)
pub async fn admin_state_delete(
    admin_user: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Extension(session): Extension<Session>,
    Form(form): Form<CsrfOnlyForm>,
) -> impl IntoResponse {
    if !validate_csrf(&session, &form.csrf_token).await {
        let template = AdminErrorTemplate {
            error_code: 403,
            error_message: "Invalid CSRF token".to_string(),
            current_admin: Some(admin_user.username),
        };
        return (StatusCode::FORBIDDEN, template).into_response();
    }

    if let Ok(count) = db::count_users_by_state_id(&state.db, id).await {
        if count > 0 {
            let template = AdminErrorTemplate {
                error_code: 400,
                error_message: "Cannot delete state assigned to users.".to_string(),
                current_admin: Some(admin_user.username),
            };
            return (StatusCode::BAD_REQUEST, template).into_response();
        }
    }

    if let Err(_) = db::delete_state(&state.db, id).await {
        let template = AdminErrorTemplate {
            error_code: 500,
            error_message: "Failed to delete state.".to_string(),
            current_admin: Some(admin_user.username),
        };
        return (StatusCode::INTERNAL_SERVER_ERROR, template).into_response();
    }

    invalidate_geo_cache(&state).await;
    Redirect::to("/admin/states").into_response()
}

// Create user page (GET) - requires authentication
pub async fn user_create_page(
    admin_user: AdminUser,
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
) -> impl IntoResponse {
    let csrf_token = ensure_csrf_token(&session).await;
    let countries = get_countries_cached(&state).await.unwrap_or_default();
    let states: Vec<StateOption> = Vec::new();
    let template = AdminCreateUserTemplate {
        error: None,
        success: None,
        username: None,
        email: None,
        current_admin: Some(admin_user.username),
        csrf_token,
        countries,
        states,
        selected_country_id: 0,
        selected_state_id: 0,
        address: None,
    };

    template
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserForm {
    #[validate(length(min = 1))]
    username: String,
    #[validate(email)]
    email: String,
    #[validate(length(min = 6))]
    password: String,
    #[validate(length(min = 1))]
    address: String,
    #[validate(range(min = 1))]
    country_id: i32,
    #[validate(range(min = 1))]
    state_id: i32,
    #[validate(length(min = 1))]
    csrf_token: String,
}

// Create user submission (POST) - requires authentication
pub async fn user_create_submit(
    admin_user: AdminUser,
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Form(new_user): Form<CreateUserForm>,
) -> impl IntoResponse {
    if !validate_csrf(&session, &new_user.csrf_token).await {
        let countries = get_countries_cached(&state).await.unwrap_or_default();
        let states = if new_user.country_id > 0 {
            get_states_cached(&state, new_user.country_id).await.unwrap_or_default()
        } else {
            Vec::new()
        };
        let template = AdminCreateUserTemplate {
            error: Some("Invalid CSRF token".to_string()),
            success: None,
            username: Some(new_user.username),
            email: Some(new_user.email),
            current_admin: Some(admin_user.username),
            csrf_token: ensure_csrf_token(&session).await,
            countries,
            states,
            selected_country_id: new_user.country_id,
            selected_state_id: new_user.state_id,
            address: Some(new_user.address),
        };
        return template.into_response();
    }

    if new_user.validate().is_err() {
        let countries = get_countries_cached(&state).await.unwrap_or_default();
        let states = if new_user.country_id > 0 {
            get_states_cached(&state, new_user.country_id).await.unwrap_or_default()
        } else {
            Vec::new()
        };
        let template = AdminCreateUserTemplate {
            error: Some("Invalid user data".to_string()),
            success: None,
            username: Some(new_user.username),
            email: Some(new_user.email),
            current_admin: Some(admin_user.username),
            csrf_token: ensure_csrf_token(&session).await,
            countries,
            states,
            selected_country_id: new_user.country_id,
            selected_state_id: new_user.state_id,
            address: Some(new_user.address),
        };
        return template.into_response();
    }

    match db::create_user(
        &state.db,
        &new_user.username,
        &new_user.email,
        &new_user.password,
        Some(new_user.address.as_str()),
        Some(new_user.country_id),
        Some(new_user.state_id),
    )
    .await
    {
        Ok(user_id) => Redirect::to(&format!("/admin/users/{}", user_id)).into_response(),
        Err(e) => {
            let error_msg = if e.to_string().contains("Duplicate entry") {
                "Username or email already exists".to_string()
            } else {
                "Failed to create user. Please try again.".to_string()
            };

            let template = AdminCreateUserTemplate {
                error: Some(error_msg),
                success: None,
                username: Some(new_user.username),
                email: Some(new_user.email),
                current_admin: Some(admin_user.username),
                csrf_token: ensure_csrf_token(&session).await,
                countries: get_countries_cached(&state).await.unwrap_or_default(),
                states: if new_user.country_id > 0 {
                    get_states_cached(&state, new_user.country_id).await.unwrap_or_default()
                } else {
                    Vec::new()
                },
                selected_country_id: new_user.country_id,
                selected_state_id: new_user.state_id,
                address: Some(new_user.address),
            };
            template.into_response()
        }
    }
}

// User detail handler - requires authentication
pub async fn user_detail(
    admin_user: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Extension(session): Extension<Session>,
) -> impl IntoResponse {
    match db::find_user_by_id(&state.db, id as i32).await {
        Ok(Some(user)) => {
            let countries = get_countries_cached(&state).await.unwrap_or_default();
            let states = if let Some(cid) = user.country_id {
                get_states_cached(&state, cid).await.unwrap_or_default()
            } else {
                Vec::new()
            };

            let country_name = user
                .country_id
                .and_then(|cid| countries.iter().find(|c| c.id == cid).map(|c| c.name.clone()));
            let state_name = user
                .state_id
                .and_then(|sid| states.iter().find(|s| s.id == sid).map(|s| s.name.clone()));

            let template = AdminUserDetailTemplate {
                user: User {
                    id: user.id as u32,
                    name: user.username,
                    email: user.email,
                    address: user.address,
                    country: country_name,
                    state: state_name,
                },
                current_admin: Some(admin_user.username),
                csrf_token: ensure_csrf_token(&session).await,
            };

            template.into_response()
        }
        Ok(None) => {
            let template = AdminErrorTemplate {
                error_code: 404,
                error_message: "User not found".to_string(),
                current_admin: Some(admin_user.username),
            };
            (StatusCode::NOT_FOUND, template).into_response()
        }
        Err(_) => {
            let template = AdminErrorTemplate {
                error_code: 500,
                error_message: "Database error. Please try again.".to_string(),
                current_admin: Some(admin_user.username),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, template).into_response()
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginForm {
    #[validate(length(min = 1))]
    username: String,
    #[validate(length(min = 6))]
    password: String,
    #[validate(length(min = 1))]
    csrf_token: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterForm {
    #[validate(length(min = 1))]
    username: String,
    #[validate(email)]
    email: String,
    #[validate(length(min = 6))]
    password: String,
    #[validate(length(min = 1))]
    csrf_token: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CsrfOnlyForm {
    #[validate(length(min = 1))]
    csrf_token: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserForm {
    #[validate(length(min = 1))]
    username: String,
    #[validate(email)]
    email: String,
    #[validate(length(min = 1))]
    address: String,
    #[validate(range(min = 1))]
    country_id: i32,
    #[validate(range(min = 1))]
    state_id: i32,
    #[serde(default)]
    new_password: String,
    #[validate(length(min = 1))]
    csrf_token: String,
}

// Edit user page (GET) - requires authentication
pub async fn user_edit_page(
    admin_user: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Extension(session): Extension<Session>,
) -> impl IntoResponse {
    match db::find_user_by_id(&state.db, id as i32).await {
        Ok(Some(user)) => {
            let countries = get_countries_cached(&state).await.unwrap_or_default();
            let states = if let Some(cid) = user.country_id {
                get_states_cached(&state, cid).await.unwrap_or_default()
            } else {
                Vec::new()
            };

            AdminEditUserTemplate {
                error: None,
                success: None,
                user_id: user.id,
                username: user.username,
                email: user.email,
                current_admin: Some(admin_user.username),
                csrf_token: ensure_csrf_token(&session).await,
                countries,
                states,
                selected_country_id: user.country_id.unwrap_or(0),
                selected_state_id: user.state_id.unwrap_or(0),
                address: user.address,
            }
            .into_response()
        }
        Ok(None) => {
            let template = AdminErrorTemplate {
                error_code: 404,
                error_message: "User not found".to_string(),
                current_admin: Some(admin_user.username),
            };
            (StatusCode::NOT_FOUND, template).into_response()
        }
        Err(_) => {
            let template = AdminErrorTemplate {
                error_code: 500,
                error_message: "Database error. Please try again.".to_string(),
                current_admin: Some(admin_user.username),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, template).into_response()
        }
    }
}

// Edit user submission (POST) - requires authentication
pub async fn user_edit_submit(
    admin_user: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Extension(session): Extension<Session>,
    Form(form): Form<UpdateUserForm>,
) -> impl IntoResponse {
    if !validate_csrf(&session, &form.csrf_token).await {
        let countries = get_countries_cached(&state).await.unwrap_or_default();
        let states = if form.country_id > 0 {
            get_states_cached(&state, form.country_id).await.unwrap_or_default()
        } else {
            Vec::new()
        };
        let template = AdminEditUserTemplate {
            error: Some("Invalid CSRF token".to_string()),
            success: None,
            user_id: id as i32,
            username: form.username,
            email: form.email,
            current_admin: Some(admin_user.username),
            csrf_token: ensure_csrf_token(&session).await,
            countries,
            states,
            selected_country_id: form.country_id,
            selected_state_id: form.state_id,
            address: Some(form.address),
        };
        return template.into_response();
    }

    if form.validate().is_err() {
        let countries = get_countries_cached(&state).await.unwrap_or_default();
        let states = if form.country_id > 0 {
            get_states_cached(&state, form.country_id).await.unwrap_or_default()
        } else {
            Vec::new()
        };
        let template = AdminEditUserTemplate {
            error: Some("Invalid user data".to_string()),
            success: None,
            user_id: id as i32,
            username: form.username,
            email: form.email,
            current_admin: Some(admin_user.username),
            csrf_token: ensure_csrf_token(&session).await,
            countries,
            states,
            selected_country_id: form.country_id,
            selected_state_id: form.state_id,
            address: Some(form.address),
        };
        return template.into_response();
    }

    let update_result = db::update_user(
        &state.db,
        id as i32,
        &form.username,
        &form.email,
        Some(form.address.as_str()),
        Some(form.country_id),
        Some(form.state_id),
    )
    .await;
    if let Err(e) = update_result {
        let error_msg = if e.to_string().contains("Duplicate entry") {
            "Username or email already exists".to_string()
        } else {
            "Failed to update user. Please try again.".to_string()
        };
        let countries = get_countries_cached(&state).await.unwrap_or_default();
        let states = if form.country_id > 0 {
            get_states_cached(&state, form.country_id).await.unwrap_or_default()
        } else {
            Vec::new()
        };
        let template = AdminEditUserTemplate {
            error: Some(error_msg),
            success: None,
            user_id: id as i32,
            username: form.username,
            email: form.email,
            current_admin: Some(admin_user.username),
            csrf_token: ensure_csrf_token(&session).await,
            countries,
            states,
            selected_country_id: form.country_id,
            selected_state_id: form.state_id,
            address: Some(form.address),
        };
        return template.into_response();
    }

    if !form.new_password.is_empty() {
        if form.new_password.len() < 6 {
            let countries = get_countries_cached(&state).await.unwrap_or_default();
            let states = if form.country_id > 0 {
                get_states_cached(&state, form.country_id).await.unwrap_or_default()
            } else {
                Vec::new()
            };
            let template = AdminEditUserTemplate {
                error: Some("New password must be at least 6 characters".to_string()),
                success: None,
                user_id: id as i32,
                username: form.username,
                email: form.email,
                current_admin: Some(admin_user.username),
                csrf_token: ensure_csrf_token(&session).await,
                countries,
                states,
                selected_country_id: form.country_id,
                selected_state_id: form.state_id,
                address: Some(form.address),
            };
            return template.into_response();
        }

        if let Err(_) = db::update_password(&state.db, id as i32, &form.new_password).await {
            let countries = get_countries_cached(&state).await.unwrap_or_default();
            let states = if form.country_id > 0 {
                get_states_cached(&state, form.country_id).await.unwrap_or_default()
            } else {
                Vec::new()
            };
            let template = AdminEditUserTemplate {
                error: Some("Failed to update password. Please try again.".to_string()),
                success: None,
                user_id: id as i32,
                username: form.username,
                email: form.email,
                current_admin: Some(admin_user.username),
                csrf_token: ensure_csrf_token(&session).await,
                countries,
                states,
                selected_country_id: form.country_id,
                selected_state_id: form.state_id,
                address: Some(form.address),
            };
            return template.into_response();
        }
    }

    AdminEditUserTemplate {
        error: None,
        success: Some("User updated successfully".to_string()),
        user_id: id as i32,
        username: form.username,
        email: form.email,
        current_admin: Some(admin_user.username),
        csrf_token: ensure_csrf_token(&session).await,
        countries: get_countries_cached(&state).await.unwrap_or_default(),
        states: if form.country_id > 0 {
            get_states_cached(&state, form.country_id).await.unwrap_or_default()
        } else {
            Vec::new()
        },
        selected_country_id: form.country_id,
        selected_state_id: form.state_id,
        address: Some(form.address),
    }
    .into_response()
}

// Delete user (POST) - requires authentication
pub async fn user_delete(
    _admin_user: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Extension(session): Extension<Session>,
    Form(form): Form<CsrfOnlyForm>,
) -> impl IntoResponse {
    if !validate_csrf(&session, &form.csrf_token).await {
        let template = AdminErrorTemplate {
            error_code: 403,
            error_message: "Invalid CSRF token".to_string(),
            current_admin: None,
        };
        return (StatusCode::FORBIDDEN, template).into_response();
    }

    if let Err(_) = db::delete_user(&state.db, id as i32).await {
        let template = AdminErrorTemplate {
            error_code: 500,
            error_message: "Failed to delete user. Please try again.".to_string(),
            current_admin: None,
        };
        return (StatusCode::INTERNAL_SERVER_ERROR, template).into_response();
    }

    Redirect::to("/admin/users").into_response()
}

// Admin login page (GET)
pub async fn admin_login_page(
    OptionalAdminUser(user): OptionalAdminUser,
    Extension(session): Extension<Session>,
) -> impl IntoResponse {
    let csrf_token = ensure_csrf_token(&session).await;
    if user.is_some() {
        return Redirect::to("/admin/users").into_response();
    }

    AdminLoginTemplate {
        error: None,
        csrf_token,
    }
    .into_response()
}

// Admin login form submission (POST)
pub async fn admin_login_submit(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Form(credentials): Form<LoginForm>,
) -> impl IntoResponse {
    if !validate_csrf(&session, &credentials.csrf_token).await {
        return AdminLoginTemplate {
            error: Some("Invalid CSRF token".to_string()),
            csrf_token: ensure_csrf_token(&session).await,
        }
        .into_response();
    }

    if credentials.validate().is_err() {
        return AdminLoginTemplate {
            error: Some("Invalid login data".to_string()),
            csrf_token: ensure_csrf_token(&session).await,
        }
        .into_response();
    }

    match db::find_admin_by_username(&state.db, &credentials.username).await {
        Ok(Some(admin)) => {
            if db::verify_password_hash(&admin.password_hash, &credentials.password).await {
                let admin_user = AdminUser::new(admin.id, admin.username);
                if let Err(_) = admin_user.login(&session).await {
                    return AdminLoginTemplate {
                        error: Some("Session error. Please try again.".to_string()),
                        csrf_token: ensure_csrf_token(&session).await,
                    }
                    .into_response();
                }

                Redirect::to("/admin/users").into_response()
            } else {
                AdminLoginTemplate {
                    error: Some("Invalid username or password".to_string()),
                    csrf_token: ensure_csrf_token(&session).await,
                }
                .into_response()
            }
        }
        Ok(None) => AdminLoginTemplate {
            error: Some("Invalid username or password".to_string()),
            csrf_token: ensure_csrf_token(&session).await,
        }
        .into_response(),
        Err(_) => AdminLoginTemplate {
            error: Some("Database error. Please try again.".to_string()),
            csrf_token: ensure_csrf_token(&session).await,
        }
        .into_response(),
    }
}

// Admin logout
pub async fn admin_logout(Extension(session): Extension<Session>) -> impl IntoResponse {
    let _ = AdminUser::logout(&session).await;
    Redirect::to("/admin/login")
}

// Admin index (GET)
pub async fn admin_index(_admin_user: AdminUser) -> impl IntoResponse {
    Redirect::to("/admin/users")
}

// Login page (GET)
pub async fn login_page(
    OptionalAuthUser(user): OptionalAuthUser,
    Extension(session): Extension<Session>,
) -> impl IntoResponse {
    let csrf_token = ensure_csrf_token(&session).await;
    // If already logged in, redirect to home
    if user.is_some() {
        return Redirect::to("/").into_response();
    }

    let template = LoginTemplate {
        error: None,
        csrf_token,
    };
    template.into_response()
}

// Login form submission (POST)
pub async fn login_submit(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Form(credentials): Form<LoginForm>,
) -> impl IntoResponse {
    if !validate_csrf(&session, &credentials.csrf_token).await {
        let template = LoginTemplate {
            error: Some("Invalid CSRF token".to_string()),
            csrf_token: ensure_csrf_token(&session).await,
        };
        return template.into_response();
    }

    if credentials.validate().is_err() {
        let template = LoginTemplate {
            error: Some("Invalid login data".to_string()),
            csrf_token: ensure_csrf_token(&session).await,
        };
        return template.into_response();
    }

    // Find user by username
    match db::find_user_by_username(&state.db, &credentials.username).await {
        Ok(Some(user)) => {
            // Verify password
            if db::verify_password_hash(&user.password_hash, &credentials.password).await {
                // Create auth user and login
                let auth_user = AuthUser::new(user.id, user.username);
                if let Err(_) = auth_user.login(&session).await {
                    let template = LoginTemplate {
                        error: Some("Session error. Please try again.".to_string()),
                        csrf_token: ensure_csrf_token(&session).await,
                    };
                    return template.into_response();
                }

                return Redirect::to("/").into_response();
            } else {
                let template = LoginTemplate {
                    error: Some("Invalid username or password".to_string()),
                    csrf_token: ensure_csrf_token(&session).await,
                };
                return template.into_response();
            }
        }
        Ok(None) => {
            let template = LoginTemplate {
                error: Some("Invalid username or password".to_string()),
                csrf_token: ensure_csrf_token(&session).await,
            };
            return template.into_response();
        }
        Err(_) => {
            let template = LoginTemplate {
                error: Some("Database error. Please try again.".to_string()),
                csrf_token: ensure_csrf_token(&session).await,
            };
            return template.into_response();
        }
    }
}

// Register page (GET)
pub async fn register_page(
    OptionalAuthUser(user): OptionalAuthUser,
    Extension(session): Extension<Session>,
) -> impl IntoResponse {
    let csrf_token = ensure_csrf_token(&session).await;
    // If already logged in, redirect to home
    if user.is_some() {
        return Redirect::to("/").into_response();
    }

    let template = RegisterTemplate {
        error: None,
        csrf_token,
    };
    template.into_response()
}

// Register form submission (POST)
pub async fn register_submit(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Form(new_user): Form<RegisterForm>,
) -> impl IntoResponse {
    if !validate_csrf(&session, &new_user.csrf_token).await {
        let template = RegisterTemplate {
            error: Some("Invalid CSRF token".to_string()),
            csrf_token: ensure_csrf_token(&session).await,
        };
        return template.into_response();
    }

    if new_user.validate().is_err() {
        let template = RegisterTemplate {
            error: Some("Invalid registration data".to_string()),
            csrf_token: ensure_csrf_token(&session).await,
        };
        return template.into_response();
    }

    // Create user
    match db::create_user(
        &state.db,
        &new_user.username,
        &new_user.email,
        &new_user.password,
        None,
        None,
        None,
    )
    .await
    {
        Ok(user_id) => {
            // Auto-login after registration
            let auth_user = AuthUser::new(user_id, new_user.username);
            if let Err(_) = auth_user.login(&session).await {
                let template = RegisterTemplate {
                    error: Some(
                        "Registration successful but login failed. Please login manually."
                            .to_string(),
                    ),
                    csrf_token: ensure_csrf_token(&session).await,
                };
                return template.into_response();
            }

            let _ = session
                .insert("flash_success", "Registration successful!".to_string())
                .await;
            Redirect::to("/").into_response()
        }
        Err(e) => {
            let error_msg = if e.to_string().contains("Duplicate entry") {
                "Username or email already exists".to_string()
            } else {
                "Registration failed. Please try again.".to_string()
            };

            let template = RegisterTemplate {
                error: Some(error_msg),
                csrf_token: ensure_csrf_token(&session).await,
            };
            template.into_response()
        }
    }
}

// Logout handler
pub async fn logout(Extension(session): Extension<Session>) -> impl IntoResponse {
    let _ = AuthUser::logout(&session).await;
    Redirect::to("/login")
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePasswordForm {
    #[validate(length(min = 6))]
    current_password: String,
    #[validate(length(min = 6))]
    new_password: String,
    #[validate(length(min = 6))]
    confirm_password: String,
    #[validate(length(min = 1))]
    csrf_token: String,
}

// Update password page (GET) - requires authentication
pub async fn update_password_page(
    auth_user: AuthUser,
    Extension(session): Extension<Session>,
) -> impl IntoResponse {
    let template = UpdatePasswordTemplate {
        error: None,
        success: None,
        current_user: Some(auth_user.username),
        csrf_token: ensure_csrf_token(&session).await,
    };

    template
}

// Update password submission (POST) - requires authentication
pub async fn update_password_submit(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Form(form): Form<UpdatePasswordForm>,
) -> impl IntoResponse {
    if !validate_csrf(&session, &form.csrf_token).await {
        let template = UpdatePasswordTemplate {
            error: Some("Invalid CSRF token".to_string()),
            success: None,
            current_user: Some(auth_user.username),
            csrf_token: ensure_csrf_token(&session).await,
        };
        return template.into_response();
    }

    if form.validate().is_err() {
        let template = UpdatePasswordTemplate {
            error: Some("Invalid password data".to_string()),
            success: None,
            current_user: Some(auth_user.username),
            csrf_token: ensure_csrf_token(&session).await,
        };
        return template.into_response();
    }

    if form.new_password != form.confirm_password {
        let template = UpdatePasswordTemplate {
            error: Some("New password and confirmation do not match".to_string()),
            success: None,
            current_user: Some(auth_user.username),
            csrf_token: ensure_csrf_token(&session).await,
        };
        return template.into_response();
    }

    match db::find_user_by_id(&state.db, auth_user.id).await {
        Ok(Some(user)) => {
            if !db::verify_password_hash(&user.password_hash, &form.current_password).await {
                let template = UpdatePasswordTemplate {
                    error: Some("Current password is incorrect".to_string()),
                    success: None,
                    current_user: Some(auth_user.username),
                    csrf_token: ensure_csrf_token(&session).await,
                };
                return template.into_response();
            }

            match db::update_password(&state.db, auth_user.id, &form.new_password).await {
                Ok(_) => UpdatePasswordTemplate {
                    error: None,
                    success: Some("Password updated successfully".to_string()),
                    current_user: Some(auth_user.username),
                    csrf_token: ensure_csrf_token(&session).await,
                }
                .into_response(),
                Err(_) => UpdatePasswordTemplate {
                    error: Some("Failed to update password. Please try again.".to_string()),
                    success: None,
                    current_user: Some(auth_user.username),
                    csrf_token: ensure_csrf_token(&session).await,
                }
                .into_response(),
            }
        }
        Ok(None) => UpdatePasswordTemplate {
            error: Some("User not found".to_string()),
            success: None,
            current_user: Some(auth_user.username),
            csrf_token: ensure_csrf_token(&session).await,
        }
        .into_response(),
        Err(_) => UpdatePasswordTemplate {
            error: Some("Database error. Please try again.".to_string()),
            success: None,
            current_user: Some(auth_user.username),
            csrf_token: ensure_csrf_token(&session).await,
        }
        .into_response(),
    }
}

// DataTables request parameters
#[derive(Debug, Deserialize)]
pub struct DataTablesRequest {
    draw: i32,
    start: i64,
    length: i64,
    #[serde(default)]
    search: DataTablesSearch,
    #[serde(default)]
    order: Vec<DataTablesOrder>,
}

#[derive(Debug, Deserialize, Default)]
pub struct DataTablesSearch {
    #[serde(default)]
    value: String,
}

#[derive(Debug, Deserialize)]
pub struct DataTablesOrder {
    column: usize,
    dir: String,
}

#[derive(Debug, Deserialize)]
pub struct StatesQuery {
    country_id: i32,
}

// DataTables response structure
#[derive(Debug, Serialize)]
pub struct DataTablesResponse {
    draw: i32,
    #[serde(rename = "recordsTotal")]
    records_total: i64,
    #[serde(rename = "recordsFiltered")]
    records_filtered: i64,
    data: Vec<UserRow>,
}

// Simplified user data for DataTables
#[derive(Debug, Serialize)]
pub struct UserRow {
    id: i32,
    username: String,
    email: String,
    created_at: String,
}

// DataTables AJAX endpoint - requires authentication
pub async fn users_datatable_api(
    _admin_user: AdminUser,
    State(state): State<AppState>,
    Query(params): Query<DataTablesRequest>,
) -> Result<Json<DataTablesResponse>, StatusCode> {
    // Column mapping (matches DataTables column order)
    let columns = vec!["id", "username", "email", "created_at"];

    // Get order column and direction
    let (order_column, order_direction) = if let Some(order) = params.order.first() {
        let col_name = columns.get(order.column).unwrap_or(&"id");
        (col_name.to_string(), order.dir.clone())
    } else {
        ("id".to_string(), "desc".to_string())
    };

    // Prepare search term
    let search_value = if params.search.value.is_empty() {
        None
    } else {
        Some(params.search.value.clone())
    };

    // Create pagination parameters
    let pagination_params = PaginationParams {
        offset: params.start,
        limit: params.length,
        search: search_value.clone(),
        order_column,
        order_direction,
    };

    // Get total count (without filters)
    let total_count = db::get_users_count(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get filtered count (with search)
    let filtered_count = db::get_filtered_users_count(&state.db, &search_value)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get paginated users
    let users = db::get_users_paginated(&state.db, &pagination_params)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Convert to UserRow format
    let user_rows: Vec<UserRow> = users
        .into_iter()
        .map(|user| UserRow {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user
                .created_at
                .format(&Rfc3339)
                .unwrap_or_else(|_| user.created_at.to_string()),
        })
        .collect();

    // Build response
    let response = DataTablesResponse {
        draw: params.draw,
        records_total: total_count,
        records_filtered: filtered_count,
        data: user_rows,
    };

    Ok(Json(response))
}

// States list for a country (admin) - cached via Redis
pub async fn admin_states_api(
    _admin_user: AdminUser,
    State(state): State<AppState>,
    Query(params): Query<StatesQuery>,
) -> Result<Json<Vec<StateOption>>, StatusCode> {
    if params.country_id <= 0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let states = get_states_cached(&state, params.country_id).await?;
    Ok(Json(states))
}

#[derive(Debug, Deserialize)]
pub struct PdfExportParams {
    search: Option<String>,
    order_column: Option<String>,
    order_direction: Option<String>,
}

// Admin export users to PDF (GET)
pub async fn admin_users_pdf(
    _admin_user: AdminUser,
    State(state): State<AppState>,
    Query(params): Query<PdfExportParams>,
) -> impl IntoResponse {
    let order_column = params
        .order_column
        .as_deref()
        .unwrap_or("id");
    let order_direction = params
        .order_direction
        .as_deref()
        .unwrap_or("desc");

    let users = match db::get_users_for_export(&state.db, &params.search, order_column, order_direction)
        .await
    {
        Ok(users) => users,
        Err(_) => {
            let template = AdminErrorTemplate {
                error_code: 500,
                error_message: "Failed to export users. Please try again.".to_string(),
                current_admin: None,
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, template).into_response();
        }
    };

    let (doc, page1, layer1) = PdfDocument::new("Users", Mm(210.0), Mm(297.0), "Layer 1");
    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .unwrap();

    let mut current_page = page1;
    let mut current_layer = layer1;
    let mut y = 280.0;
    let left_margin = 15.0;
    let line_height = 6.0;

    {
        let layer = doc.get_page(current_page).get_layer(current_layer);
        layer.use_text("Users", 16.0, Mm(left_margin), Mm(y), &font);
    }
    y -= 10.0;

    for user in users {
        if y < 15.0 {
            let (page, layer) = doc.add_page(Mm(210.0), Mm(297.0), "Layer");
            current_page = page;
            current_layer = layer;
            y = 280.0;
        }

        let created_at = user
            .created_at
            .format(&Rfc3339)
            .unwrap_or_else(|_| user.created_at.to_string());
        let line = format!(
            "{} | {} | {} | {}",
            user.id, user.username, user.email, created_at
        );
        let layer = doc.get_page(current_page).get_layer(current_layer);
        layer.use_text(line, 9.0, Mm(left_margin), Mm(y), &font);
        y -= line_height;
    }

    let mut pdf_bytes: Vec<u8> = Vec::new();
    {
        let mut writer = BufWriter::new(&mut pdf_bytes);
        if doc.save(&mut writer).is_err() {
            let template = AdminErrorTemplate {
                error_code: 500,
                error_message: "Failed to generate PDF. Please try again.".to_string(),
                current_admin: None,
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, template).into_response();
        }
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/pdf"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"users.pdf\""),
    );

    (StatusCode::OK, headers, pdf_bytes).into_response()
}

// Error handler
pub async fn handle_404() -> impl IntoResponse {
    let template = ErrorTemplate {
        error_code: 404,
        error_message: "Page not found".to_string(),
    };

    (StatusCode::NOT_FOUND, template)
}
