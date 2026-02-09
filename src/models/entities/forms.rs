use serde::Deserialize;

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
pub struct CsrfOnlyForm {
    #[validate(length(min = 1))]
    pub csrf_token: String,
}