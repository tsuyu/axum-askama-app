use askama::Template;
use crate::filters;

// Index page template
#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub title: String,
    pub message: String,
    pub user: Option<String>,
    pub flash_success: Option<String>,
}

// Error page template
#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate {
    pub error_code: u16,
    pub error_message: String,
}

// Login page template
#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub error: Option<String>,
    pub csrf_token: String,
}

// Register page template
#[derive(Template)]
#[template(path = "register.html")]
pub struct RegisterTemplate {
    pub error: Option<String>,
    pub csrf_token: String,
}

// Update password page template
#[derive(Template)]
#[template(path = "users/password.html")]
pub struct UpdatePasswordTemplate {
    pub error: Option<String>,
    pub success: Option<String>,
    pub current_user: Option<String>,
    pub csrf_token: String,
}

// User dashboard template
#[derive(Template)]
#[template(path = "users/dashboard.html")]
pub struct UserDashboardTemplate {
    pub current_user: Option<String>,
}

