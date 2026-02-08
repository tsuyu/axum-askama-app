use axum::{
    Form,
    extract::{Extension, Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Json, Redirect},
};
use printpdf::{BuiltinFont, Mm, PdfDocument};
use std::io::BufWriter;
use time::format_description::well_known::Rfc3339;
use tower_sessions::Session;
use validator::Validate;

use crate::controllers::auth_controller::{AdminUser, OptionalAdminUser};
use crate::models::db;
use crate::state::AppState;
use crate::views::templates::{
    AdminCreateUserTemplate, AdminEditUserTemplate, AdminErrorTemplate, AdminLoginTemplate,
    AdminUserDetailTemplate, AdminUsersListTemplate, AdminCountriesListTemplate,
    AdminCountryFormTemplate, AdminStatesListTemplate, AdminStateFormTemplate, AdminStateRow,
    User, CountryOption, StateOption,
};

use super::shared::{
    ensure_csrf_token, validate_csrf, map_country_options, get_countries_cached,
    get_states_cached, invalidate_geo_cache, CountryForm, StateForm, CreateUserForm,
    LoginForm, CsrfOnlyForm, UpdateUserForm, StatesQuery, PdfExportParams,
};

// Users list handler - requires authentication
pub async fn users_list(admin_user: AdminUser) -> impl IntoResponse {
    tracing::info!("Admin {} accessed users list", admin_user.username);
    let template = AdminUsersListTemplate {
        page_title: "All Users".to_string(),
        current_admin: Some(admin_user.username),
    };

    template
}

// Users datatable API for AJAX requests
#[derive(serde::Serialize)]
pub struct DatatableResponse<T> {
    draw: u32,
    #[serde(rename = "recordsTotal")]
    records_total: i64,
    #[serde(rename = "recordsFiltered")]
    records_filtered: i64,
    data: Vec<T>,
}

#[derive(serde::Deserialize)]
pub struct DatatableParams {
    draw: u32,
    start: Option<i64>,
    length: Option<i64>,
    #[serde(rename = "search[value]")]
    search_value: Option<String>,
    #[serde(rename = "order[0][column]")]
    order_column: Option<i32>,
    #[serde(rename = "order[0][dir]")]
    order_dir: Option<String>,
}

pub async fn users_datatable_api(
    admin_user: AdminUser,
    State(state): State<AppState>,
    Query(params): Query<DatatableParams>,
) -> impl IntoResponse {
    tracing::debug!("Admin {} requested users datatable", admin_user.username);
    
    let draw = params.draw;
    let offset = params.start.unwrap_or(0);
    let limit = params.length.unwrap_or(10);
    let search = params.search_value.filter(|s| !s.is_empty());

    // Get total count
    let total_count = match db::get_users_count(&state.db).await {
        Ok(count) => count,
        Err(e) => {
            tracing::error!("Failed to get users count: {:?}", e);
            return Json(DatatableResponse {
                draw,
                records_total: 0,
                records_filtered: 0,
                data: Vec::<db::User>::new(),
            });
        }
    };

    // Map column index to column name
    let order_column = match params.order_column {
        Some(0) => "id",
        Some(1) => "username",
        Some(2) => "email",
        Some(3) => "created_at",
        _ => "id",
    };

    let order_direction = match params.order_dir.as_deref() {
        Some("asc") => "asc",
        _ => "desc",
    };

    let pagination_params = db::PaginationParams {
        offset,
        limit,
        search,
        order_column: order_column.to_string(),
        order_direction: order_direction.to_string(),
    };

    // Get paginated users
    match db::get_users_paginated(&state.db, &pagination_params).await {
        Ok(users) => {
            let filtered_count = users.len() as i64;
            tracing::info!(
                "Admin {} retrieved {} users",
                admin_user.username,
                filtered_count
            );
            Json(DatatableResponse {
                draw,
                records_total: total_count,
                records_filtered: filtered_count,
                data: users,
            })
        }
        Err(e) => {
            tracing::error!("Failed to get paginated users: {:?}", e);
            Json(DatatableResponse {
                draw,
                records_total: total_count,
                records_filtered: 0,
                data: Vec::<db::User>::new(),
            })
        }
    }
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

// Admin index
pub async fn admin_index(OptionalAdminUser(admin_user): OptionalAdminUser) -> impl IntoResponse {
    if admin_user.is_some() {
        Redirect::to("/admin/users").into_response()
    } else {
        Redirect::to("/admin/login").into_response()
    }
}

// Admin login page (GET)
pub async fn admin_login_page(
    OptionalAdminUser(admin_user): OptionalAdminUser,
    Extension(session): Extension<Session>,
) -> impl IntoResponse {
    if admin_user.is_some() {
        return Redirect::to("/admin/users").into_response();
    }

    AdminLoginTemplate {
        error: None,
        csrf_token: ensure_csrf_token(&session).await,
    }
    .into_response()
}

// Admin login submission (POST)
pub async fn admin_login_submit(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Form(credentials): Form<LoginForm>,
) -> impl IntoResponse {
    tracing::debug!("Admin login attempt for user: {}", credentials.username);
    
    if !validate_csrf(&session, &credentials.csrf_token).await {
        tracing::warn!("Admin login failed: Invalid CSRF token");
        return AdminLoginTemplate {
            error: Some("Invalid CSRF token".to_string()),
            csrf_token: ensure_csrf_token(&session).await,
        }
        .into_response();
    }

    if credentials.validate().is_err() {
        tracing::warn!("Admin login failed: Invalid login data");
        return AdminLoginTemplate {
            error: Some("Invalid login data".to_string()),
            csrf_token: ensure_csrf_token(&session).await,
        }
        .into_response();
    }

    match db::find_admin_by_username(&state.db, &credentials.username).await {
        Ok(Some(admin)) => {
            tracing::debug!("Admin user found: {}", admin.username);
            if db::verify_password_hash(&admin.password_hash, &credentials.password).await {
                tracing::info!("Admin login successful: {}", admin.username);
                let admin_user = AdminUser::new(admin.id, admin.username.clone());
                if let Err(e) = admin_user.login(&session).await {
                    tracing::error!("Failed to set admin session: {:?}", e);
                    return AdminLoginTemplate {
                        error: Some("Session error. Please try again.".to_string()),
                        csrf_token: ensure_csrf_token(&session).await,
                    }
                    .into_response();
                }

                tracing::info!("Admin session created, redirecting to /admin/users");
                Redirect::to("/admin/users").into_response()
            } else {
                tracing::warn!("Admin login failed: Invalid password for {}", credentials.username);
                AdminLoginTemplate {
                    error: Some("Invalid username or password".to_string()),
                    csrf_token: ensure_csrf_token(&session).await,
                }
                .into_response()
            }
        }
        Ok(None) => {
            tracing::warn!("Admin login failed: User not found - {}", credentials.username);
            AdminLoginTemplate {
                error: Some("Invalid username or password".to_string()),
                csrf_token: ensure_csrf_token(&session).await,
            }
            .into_response()
        }
        Err(e) => {
            tracing::error!("Admin login database error: {:?}", e);
            AdminLoginTemplate {
                error: Some("Database error. Please try again.".to_string()),
                csrf_token: ensure_csrf_token(&session).await,
            }
            .into_response()
        }
    }
}

// Admin logout
pub async fn admin_logout(Extension(session): Extension<Session>) -> impl IntoResponse {
    let _ = AdminUser::logout(&session).await;
    Redirect::to("/admin/login")
}

// States list (admin)
pub async fn admin_states_list(
    admin_user: AdminUser,
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
) -> impl IntoResponse {
    let rows = match db::get_states_with_countries(&state.db).await {
        Ok(rows) => rows,
        Err(_) => {
            let template = AdminErrorTemplate {
                error_code: 500,
                error_message: "Failed to load states.".to_string(),
                current_admin: Some(admin_user.username),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, template).into_response();
        }
    };

    let states = rows
        .into_iter()
        .map(|row| AdminStateRow {
            id: row.id,
            country_id: row.country_id,
            country_name: row.country_name,
            name: row.name,
        })
        .collect();

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
    let countries = match get_countries_cached(&state).await {
        Ok(countries) => countries,
        Err(code) => {
            let template = AdminErrorTemplate {
                error_code: code.as_u16(),
                error_message: "Failed to load countries.".to_string(),
                current_admin: Some(admin_user.username),
            };
            return (code, template).into_response();
        }
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
    let countries = match get_countries_cached(&state).await {
        Ok(countries) => countries,
        Err(code) => {
            let template = AdminErrorTemplate {
                error_code: code.as_u16(),
                error_message: "Failed to load countries.".to_string(),
                current_admin: Some(admin_user.username),
            };
            return (code, template).into_response();
        }
    };

    if !validate_csrf(&session, &form.csrf_token).await {
        return AdminStateFormTemplate {
            form_title: "Create State".to_string(),
            form_action: "/admin/states".to_string(),
            submit_label: "Create State".to_string(),
            state_id: None,
            name: Some(form.name.clone()),
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
        return AdminStateFormTemplate {
            form_title: "Create State".to_string(),
            form_action: "/admin/states".to_string(),
            submit_label: "Create State".to_string(),
            state_id: None,
            name: Some(form.name.clone()),
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
        return AdminStateFormTemplate {
            form_title: "Create State".to_string(),
            form_action: "/admin/states".to_string(),
            submit_label: "Create State".to_string(),
            state_id: None,
            name: Some(form.name.clone()),
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

    let countries = match get_countries_cached(&state).await {
        Ok(countries) => countries,
        Err(code) => {
            let template = AdminErrorTemplate {
                error_code: code.as_u16(),
                error_message: "Failed to load countries.".to_string(),
                current_admin: Some(admin_user.username),
            };
            return (code, template).into_response();
        }
    };

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
    let countries = match get_countries_cached(&state).await {
        Ok(countries) => countries,
        Err(code) => {
            let template = AdminErrorTemplate {
                error_code: code.as_u16(),
                error_message: "Failed to load countries.".to_string(),
                current_admin: Some(admin_user.username),
            };
            return (code, template).into_response();
        }
    };

    if !validate_csrf(&session, &form.csrf_token).await {
        return AdminStateFormTemplate {
            form_title: "Edit State".to_string(),
            form_action: format!("/admin/states/{}", id),
            submit_label: "Save Changes".to_string(),
            state_id: Some(id),
            name: Some(form.name.clone()),
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
        return AdminStateFormTemplate {
            form_title: "Edit State".to_string(),
            form_action: format!("/admin/states/{}", id),
            submit_label: "Save Changes".to_string(),
            state_id: Some(id),
            name: Some(form.name.clone()),
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
        return AdminStateFormTemplate {
            form_title: "Edit State".to_string(),
            form_action: format!("/admin/states/{}", id),
            submit_label: "Save Changes".to_string(),
            state_id: Some(id),
            name: Some(form.name.clone()),
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

// States API (admin)
pub async fn admin_states_api(
    _admin_user: AdminUser,
    State(state): State<AppState>,
    Query(query): Query<StatesQuery>,
) -> impl IntoResponse {
    match get_states_cached(&state, query.country_id).await {
        Ok(states) => Json(states).into_response(),
        Err(code) => code.into_response(),
    }
}

// Users PDF export
pub async fn admin_users_pdf(
    _admin_user: AdminUser,
    State(state): State<AppState>,
    Query(params): Query<PdfExportParams>,
) -> impl IntoResponse {
    let order_column = params.order_column.as_deref().unwrap_or("id");
    let order_direction = params.order_direction.as_deref().unwrap_or("desc");

    let users = match db::get_users_for_export(&state.db, &params.search, order_column, order_direction).await {
        Ok(users) => users,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let (doc, page1, layer1) = PdfDocument::new("Users", Mm(210.0), Mm(297.0), "Layer 1");
    let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();

    let mut current_page = page1;
    let mut current_layer = layer1;
    let mut y = 280.0;

    let header = "ID    Username                Email                           Created At";
    let mut layer = doc.get_page(current_page).get_layer(current_layer);
    layer.use_text(header, 12.0, Mm(10.0), Mm(y), &font);
    y -= 10.0;

    for user in users {
        if y < 20.0 {
            let (page, layer_id) = doc.add_page(Mm(210.0), Mm(297.0), "Layer");
            current_page = page;
            current_layer = layer_id;
            layer = doc.get_page(current_page).get_layer(current_layer);
            y = 280.0;
        }

        let created_at = user.created_at.format(&Rfc3339).unwrap_or_default();
        let line = format!(
            "{:<5} {:<22} {:<30} {}",
            user.id, user.username, user.email, created_at
        );
        layer.use_text(line, 10.0, Mm(10.0), Mm(y), &font);
        y -= 8.0;
    }

    let mut buffer = Vec::new();
    {
        let mut writer = BufWriter::new(&mut buffer);
        if doc.save(&mut writer).is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/pdf"));
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("inline; filename=\"users.pdf\""),
    );

    (headers, buffer).into_response()
}

// Admin user create page (GET)
pub async fn user_create_page(
    admin_user: AdminUser,
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
) -> impl IntoResponse {
    let countries = match get_countries_cached(&state).await {
        Ok(countries) => countries,
        Err(code) => {
            let template = AdminErrorTemplate {
                error_code: code.as_u16(),
                error_message: "Failed to load countries.".to_string(),
                current_admin: Some(admin_user.username),
            };
            return (code, template).into_response();
        }
    };

    AdminCreateUserTemplate {
        error: None,
        success: None,
        username: None,
        email: None,
        current_admin: Some(admin_user.username),
        csrf_token: ensure_csrf_token(&session).await,
        countries,
        states: Vec::new(),
        selected_country_id: 0,
        selected_state_id: 0,
        address: None,
    }
    .into_response()
}

// Admin user create submission (POST)
pub async fn user_create_submit(
    admin_user: AdminUser,
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Form(form): Form<CreateUserForm>,
) -> impl IntoResponse {
    let countries = match get_countries_cached(&state).await {
        Ok(countries) => countries,
        Err(code) => {
            let template = AdminErrorTemplate {
                error_code: code.as_u16(),
                error_message: "Failed to load countries.".to_string(),
                current_admin: Some(admin_user.username),
            };
            return (code, template).into_response();
        }
    };
    let states = match get_states_cached(&state, form.country_id).await {
        Ok(states) => states,
        Err(_) => Vec::new(),
    };

    if !validate_csrf(&session, &form.csrf_token).await {
        return AdminCreateUserTemplate {
            error: Some("Invalid CSRF token".to_string()),
            success: None,
            username: Some(form.username.clone()),
            email: Some(form.email.clone()),
            current_admin: Some(admin_user.username),
            csrf_token: ensure_csrf_token(&session).await,
            countries,
            states,
            selected_country_id: form.country_id,
            selected_state_id: form.state_id,
            address: Some(form.address.clone()),
        }
        .into_response();
    }

    if form.validate().is_err() {
        return AdminCreateUserTemplate {
            error: Some("Invalid user data".to_string()),
            success: None,
            username: Some(form.username.clone()),
            email: Some(form.email.clone()),
            current_admin: Some(admin_user.username),
            csrf_token: ensure_csrf_token(&session).await,
            countries,
            states,
            selected_country_id: form.country_id,
            selected_state_id: form.state_id,
            address: Some(form.address.clone()),
        }
        .into_response();
    }

    match db::create_user(
        &state.db,
        &form.username,
        &form.email,
        &form.password,
        Some(&form.address),
        Some(form.country_id),
        Some(form.state_id),
    )
    .await
    {
        Ok(_) => Redirect::to("/admin/users").into_response(),
        Err(e) => {
            let msg = if e.to_string().contains("Duplicate entry") {
                "Username or email already exists".to_string()
            } else {
                "Failed to create user".to_string()
            };
            AdminCreateUserTemplate {
                error: Some(msg),
                success: None,
                username: Some(form.username.clone()),
                email: Some(form.email.clone()),
                current_admin: Some(admin_user.username),
                csrf_token: ensure_csrf_token(&session).await,
                countries,
                states,
                selected_country_id: form.country_id,
                selected_state_id: form.state_id,
                address: Some(form.address.clone()),
            }
            .into_response()
        }
    }
}

// Admin user detail (GET)
pub async fn user_detail(
    admin_user: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Extension(session): Extension<Session>,
) -> impl IntoResponse {
    let user = match db::find_user_by_id(&state.db, id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            let template = AdminErrorTemplate {
                error_code: 404,
                error_message: "User not found.".to_string(),
                current_admin: Some(admin_user.username),
            };
            return (StatusCode::NOT_FOUND, template).into_response();
        }
        Err(_) => {
            let template = AdminErrorTemplate {
                error_code: 500,
                error_message: "Failed to load user.".to_string(),
                current_admin: Some(admin_user.username),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, template).into_response();
        }
    };

    let country = if let Some(country_id) = user.country_id {
        db::get_country_by_id(&state.db, country_id)
            .await
            .ok()
            .flatten()
            .map(|c| c.name)
    } else {
        None
    };

    let state_name = if let Some(state_id) = user.state_id {
        db::get_state_by_id(&state.db, state_id)
            .await
            .ok()
            .flatten()
            .map(|s| s.name)
    } else {
        None
    };

    let template_user = User {
        id: user.id as u32,
        name: user.username,
        email: user.email,
        address: user.address,
        country,
        state: state_name,
    };

    AdminUserDetailTemplate {
        user: template_user,
        current_admin: Some(admin_user.username),
        csrf_token: ensure_csrf_token(&session).await,
    }
    .into_response()
}

// Admin user edit page (GET)
pub async fn user_edit_page(
    admin_user: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Extension(session): Extension<Session>,
) -> impl IntoResponse {
    let user = match db::find_user_by_id(&state.db, id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            let template = AdminErrorTemplate {
                error_code: 404,
                error_message: "User not found.".to_string(),
                current_admin: Some(admin_user.username),
            };
            return (StatusCode::NOT_FOUND, template).into_response();
        }
        Err(_) => {
            let template = AdminErrorTemplate {
                error_code: 500,
                error_message: "Failed to load user.".to_string(),
                current_admin: Some(admin_user.username),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, template).into_response();
        }
    };

    let countries = match get_countries_cached(&state).await {
        Ok(countries) => countries,
        Err(code) => {
            let template = AdminErrorTemplate {
                error_code: code.as_u16(),
                error_message: "Failed to load countries.".to_string(),
                current_admin: Some(admin_user.username),
            };
            return (code, template).into_response();
        }
    };

    let selected_country_id = user.country_id.unwrap_or(0);
    let states = if selected_country_id > 0 {
        get_states_cached(&state, selected_country_id).await.unwrap_or_default()
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
        selected_country_id,
        selected_state_id: user.state_id.unwrap_or(0),
        address: user.address,
    }
    .into_response()
}

// Admin user edit submission (POST)
pub async fn user_edit_submit(
    admin_user: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Extension(session): Extension<Session>,
    Form(form): Form<UpdateUserForm>,
) -> impl IntoResponse {
    let countries = match get_countries_cached(&state).await {
        Ok(countries) => countries,
        Err(code) => {
            let template = AdminErrorTemplate {
                error_code: code.as_u16(),
                error_message: "Failed to load countries.".to_string(),
                current_admin: Some(admin_user.username),
            };
            return (code, template).into_response();
        }
    };
    let states = match get_states_cached(&state, form.country_id).await {
        Ok(states) => states,
        Err(_) => Vec::new(),
    };

    if !validate_csrf(&session, &form.csrf_token).await {
        return AdminEditUserTemplate {
            error: Some("Invalid CSRF token".to_string()),
            success: None,
            user_id: id,
            username: form.username.clone(),
            email: form.email.clone(),
            current_admin: Some(admin_user.username),
            csrf_token: ensure_csrf_token(&session).await,
            countries,
            states,
            selected_country_id: form.country_id,
            selected_state_id: form.state_id,
            address: Some(form.address.clone()),
        }
        .into_response();
    }

    if form.validate().is_err() {
        return AdminEditUserTemplate {
            error: Some("Invalid user data".to_string()),
            success: None,
            user_id: id,
            username: form.username.clone(),
            email: form.email.clone(),
            current_admin: Some(admin_user.username),
            csrf_token: ensure_csrf_token(&session).await,
            countries,
            states,
            selected_country_id: form.country_id,
            selected_state_id: form.state_id,
            address: Some(form.address.clone()),
        }
        .into_response();
    }

    if let Err(_) = db::update_user(
        &state.db,
        id,
        &form.username,
        &form.email,
        Some(&form.address),
        Some(form.country_id),
        Some(form.state_id),
    )
    .await
    {
        return AdminEditUserTemplate {
            error: Some("Failed to update user".to_string()),
            success: None,
            user_id: id,
            username: form.username.clone(),
            email: form.email.clone(),
            current_admin: Some(admin_user.username),
            csrf_token: ensure_csrf_token(&session).await,
            countries,
            states,
            selected_country_id: form.country_id,
            selected_state_id: form.state_id,
            address: Some(form.address.clone()),
        }
        .into_response();
    }

    if !form.new_password.trim().is_empty() {
        if let Err(_) = db::update_password(&state.db, id, &form.new_password).await {
            return AdminEditUserTemplate {
                error: Some("Failed to update password".to_string()),
                success: None,
                user_id: id,
                username: form.username.clone(),
                email: form.email.clone(),
                current_admin: Some(admin_user.username),
                csrf_token: ensure_csrf_token(&session).await,
                countries,
                states,
                selected_country_id: form.country_id,
                selected_state_id: form.state_id,
                address: Some(form.address.clone()),
            }
            .into_response();
        }
    }

    Redirect::to(&format!("/admin/users/{}", id)).into_response()
}

// Admin user delete (POST)
pub async fn user_delete(
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

    if let Err(_) = db::delete_user(&state.db, id).await {
        let template = AdminErrorTemplate {
            error_code: 500,
            error_message: "Failed to delete user.".to_string(),
            current_admin: Some(admin_user.username),
        };
        return (StatusCode::INTERNAL_SERVER_ERROR, template).into_response();
    }

    Redirect::to("/admin/users").into_response()
}
