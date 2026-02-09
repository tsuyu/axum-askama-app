use serde::Deserialize;

// Form structs for controllers
#[derive(Debug, Deserialize, validator::Validate)]
pub struct CountryForm {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(length(min = 1))]
    pub csrf_token: String,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct StateForm {
    #[validate(range(min = 1))]
    pub country_id: i32,
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(length(min = 1))]
    pub csrf_token: String,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct CreateUserForm {
    #[validate(length(min = 1))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
    #[validate(length(min = 1))]
    pub address: String,
    #[validate(range(min = 1))]
    pub country_id: i32,
    #[validate(range(min = 1))]
    pub state_id: i32,
    #[validate(length(min = 1))]
    pub csrf_token: String,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct LoginForm {
    #[validate(length(min = 1))]
    pub username: String,
    #[validate(length(min = 6))]
    pub password: String,
    #[validate(length(min = 1))]
    pub csrf_token: String,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct RegisterForm {
    #[validate(length(min = 1))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
    #[validate(length(min = 1))]
    pub csrf_token: String,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct CsrfOnlyForm {
    #[validate(length(min = 1))]
    pub csrf_token: String,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct UpdateUserForm {
    #[validate(length(min = 1))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub address: String,
    #[validate(range(min = 1))]
    pub country_id: i32,
    #[validate(range(min = 1))]
    pub state_id: i32,
    #[serde(default)]
    pub new_password: String,
    #[validate(length(min = 1))]
    pub csrf_token: String,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct UpdatePasswordForm {
    #[validate(length(min = 6))]
    pub current_password: String,
    #[validate(length(min = 6))]
    pub new_password: String,
    #[validate(length(min = 6))]
    pub confirm_password: String,
    #[validate(length(min = 1))]
    pub csrf_token: String,
}
