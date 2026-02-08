use askama::Template;
use serde::{Serialize, Deserialize};

// Re-export view data structures from entities
pub use crate::models::{UserView as User, CountryOption, StateOption, AdminStateRow};

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
#[template(path = "password.html")]
pub struct UpdatePasswordTemplate {
    pub error: Option<String>,
    pub success: Option<String>,
    pub current_user: Option<String>,
    pub csrf_token: String,
}

// Admin templates
#[derive(Template)]
#[template(path = "admin/login.html")]
pub struct AdminLoginTemplate {
    pub error: Option<String>,
    pub csrf_token: String,
}

#[derive(Template)]
#[template(path = "admin/error.html")]
pub struct AdminErrorTemplate {
    pub error_code: u16,
    pub error_message: String,
    pub current_admin: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/users/list.html")]
pub struct AdminUsersListTemplate {
    pub page_title: String,
    pub current_admin: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/users/detail.html")]
pub struct AdminUserDetailTemplate {
    pub user: User,
    pub current_admin: Option<String>,
    pub csrf_token: String,
}

#[derive(Template)]
#[template(path = "admin/users/create.html")]
pub struct AdminCreateUserTemplate {
    pub error: Option<String>,
    pub success: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub current_admin: Option<String>,
    pub csrf_token: String,
    pub countries: Vec<CountryOption>,
    pub states: Vec<StateOption>,
    pub selected_country_id: i32,
    pub selected_state_id: i32,
    pub address: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/users/edit.html")]
pub struct AdminEditUserTemplate {
    pub error: Option<String>,
    pub success: Option<String>,
    pub user_id: i32,
    pub username: String,
    pub email: String,
    pub current_admin: Option<String>,
    pub csrf_token: String,
    pub countries: Vec<CountryOption>,
    pub states: Vec<StateOption>,
    pub selected_country_id: i32,
    pub selected_state_id: i32,
    pub address: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/geo/countries_list.html")]
pub struct AdminCountriesListTemplate {
    pub page_title: String,
    pub current_admin: Option<String>,
    pub csrf_token: String,
    pub countries: Vec<CountryOption>,
}

#[derive(Template)]
#[template(path = "admin/geo/country_form.html")]
pub struct AdminCountryFormTemplate {
    pub form_title: String,
    pub form_action: String,
    pub submit_label: String,
    pub country_id: Option<i32>,
    pub name: Option<String>,
    pub error: Option<String>,
    pub success: Option<String>,
    pub current_admin: Option<String>,
    pub csrf_token: String,
}

#[derive(Template)]
#[template(path = "admin/geo/states_list.html")]
pub struct AdminStatesListTemplate {
    pub page_title: String,
    pub current_admin: Option<String>,
    pub csrf_token: String,
    pub states: Vec<AdminStateRow>,
}

#[derive(Template)]
#[template(path = "admin/geo/state_form.html")]
pub struct AdminStateFormTemplate {
    pub form_title: String,
    pub form_action: String,
    pub submit_label: String,
    pub state_id: Option<i32>,
    pub name: Option<String>,
    pub countries: Vec<CountryOption>,
    pub selected_country_id: i32,
    pub error: Option<String>,
    pub success: Option<String>,
    pub current_admin: Option<String>,
    pub csrf_token: String,
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
