use askama::Template;

use super::User;

// Index page template
#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub title: String,
    pub message: String,
    pub user: Option<String>,
    pub flash_success: Option<String>,
}

// Users list template
#[derive(Template)]
#[template(path = "users/list.html")]
pub struct UsersListTemplate {
    pub page_title: String,
    pub current_user: Option<String>,
}

// User detail template
#[derive(Template)]
#[template(path = "users/detail.html")]
pub struct UserDetailTemplate {
    pub user: User,
    pub current_user: Option<String>,
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

// Create user page template
#[derive(Template)]
#[template(path = "users/create.html")]
pub struct CreateUserTemplate {
    pub error: Option<String>,
    pub success: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub current_user: Option<String>,
}

// Edit user page template
#[derive(Template)]
#[template(path = "users/edit.html")]
pub struct EditUserTemplate {
    pub error: Option<String>,
    pub success: Option<String>,
    pub user_id: i32,
    pub username: String,
    pub email: String,
    pub current_user: Option<String>,
}
