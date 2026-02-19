use axum::{
    Form,
    extract::{Extension, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use tower_sessions::Session;
use validator::Validate;

use crate::controllers::auth_controller::{
    AdminUser, AuthUser, OptionalAdminUser, OptionalAuthUser,
};
use crate::filters;
use crate::models;
use crate::repository;
use crate::state::AppState;
use crate::utils;
use crate::views::templates::{
    AdminLoginTemplate, ErrorTemplate, IndexTemplate, LoginTemplate, RegisterTemplate,
};

use super::shared::{ensure_csrf_token, validate_csrf, LoginForm, RegisterForm};

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

    IndexTemplate {
        title: "Welcome".to_string(),
        message: "Hello from Axum + Askama!".to_string(),
        user: user.map(|u| u.username),
        flash_success,
    }
    .into_response()
}

// Login page (GET)
pub async fn login_page(
    OptionalAuthUser(user): OptionalAuthUser,
    Extension(session): Extension<Session>,
) -> impl IntoResponse {
    let csrf_token = ensure_csrf_token(&session).await;
    // If already logged in, redirect to home
    if user.is_some() {
        return Redirect::to(&filters::path("/user/dashboard")).into_response();
    }

    let template = LoginTemplate {
        error: None,
        csrf_token,
    };
    template.into_response()
}

// Admin login page (GET)
pub async fn admin_login_page(
    OptionalAdminUser(admin_user): OptionalAdminUser,
    Extension(session): Extension<Session>,
) -> impl IntoResponse {
    if admin_user.is_some() {
        return Redirect::to(&filters::path("/admin/dashboard")).into_response();
    }

    AdminLoginTemplate {
        error: None,
        csrf_token: ensure_csrf_token(&session).await,
    }
    .into_response()
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
    match repository::find_user_by_username(&state.db, &credentials.username).await {
        Ok(Some(user)) => {
            // Verify password
            if utils::verify_password_hash(&user.password_hash, &credentials.password).await {
                // Create auth user and login
                let auth_user = AuthUser::new(user.id, user.username);
                if let Err(_) = auth_user.login(&session).await {
                    let template = LoginTemplate {
                        error: Some("Session error. Please try again.".to_string()),
                        csrf_token: ensure_csrf_token(&session).await,
                    };
                    return template.into_response();
                }

                // Cycle session ID for security (prevent session fixation)
                if let Err(_) = session.cycle_id().await {
                    let template = LoginTemplate {
                        error: Some("Session error. Please try again.".to_string()),
                        csrf_token: ensure_csrf_token(&session).await,
                    };
                    return template.into_response();
                }

                return Redirect::to(&filters::path("/user/dashboard")).into_response();
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

    match repository::find_admin_by_username(&state.db, &credentials.username).await {
        Ok(Some(admin)) => {
            tracing::debug!("Admin user found: {}", admin.username);
            if utils::verify_password_hash(&admin.password_hash, &credentials.password).await {
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

                // Cycle session ID for security (prevent session fixation)
                if let Err(e) = session.cycle_id().await {
                    tracing::error!("Failed to cycle session ID: {:?}", e);
                    return AdminLoginTemplate {
                        error: Some("Session error. Please try again.".to_string()),
                        csrf_token: ensure_csrf_token(&session).await,
                    }
                    .into_response();
                }

                tracing::info!("Admin session created, redirecting to /admin/dashboard");
                Redirect::to(&filters::path("/admin/dashboard")).into_response()
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

// Register page (GET)
pub async fn register_page(
    OptionalAuthUser(user): OptionalAuthUser,
    Extension(session): Extension<Session>,
) -> impl IntoResponse {
    let csrf_token = ensure_csrf_token(&session).await;
    // If already logged in, redirect to home
    if user.is_some() {
        return Redirect::to(&filters::path("/user/dashboard")).into_response();
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
    match repository::create_user(
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

            // Cycle session ID for security (prevent session fixation)
            if let Err(_) = session.cycle_id().await {
                let template = RegisterTemplate {
                    error: Some(
                        "Registration successful but session error. Please login manually."
                            .to_string(),
                    ),
                    csrf_token: ensure_csrf_token(&session).await,
                };
                return template.into_response();
            }

            let _ = session
                .insert("flash_success", "Registration successful!".to_string())
                .await;
            Redirect::to(&filters::path("/user/dashboard")).into_response()
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
    Redirect::to(&filters::path("/login"))
}

// Error handler
pub async fn handle_404() -> impl IntoResponse {
    let template = ErrorTemplate {
        error_code: 404,
        error_message: "Page not found".to_string(),
    };

    (StatusCode::NOT_FOUND, template)
}
