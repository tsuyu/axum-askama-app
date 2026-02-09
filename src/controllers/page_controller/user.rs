use axum::{
    Form,
    extract::{Extension, State},
    response::IntoResponse,
};
use tower_sessions::Session;
use validator::Validate;

use crate::controllers::auth_controller::AuthUser;
use crate::models;
use crate::repository;
use crate::state::AppState;
use crate::utils;
use crate::views::templates::{UpdatePasswordTemplate, UserDashboardTemplate};

use super::shared::{ensure_csrf_token, validate_csrf, UpdatePasswordForm};

// User dashboard (GET) - requires authentication
pub async fn user_dashboard(auth_user: AuthUser) -> impl IntoResponse {
    UserDashboardTemplate {
        current_user: Some(auth_user.username),
    }
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

    match repository::find_user_by_id(&state.db, auth_user.id).await {
        Ok(Some(user)) => {
            if !utils::verify_password_hash(&user.password_hash, &form.current_password).await {
                let template = UpdatePasswordTemplate {
                    error: Some("Current password is incorrect".to_string()),
                    success: None,
                    current_user: Some(auth_user.username),
                    csrf_token: ensure_csrf_token(&session).await,
                };
                return template.into_response();
            }

            match repository::update_password(&state.db, auth_user.id, &form.new_password).await {
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
