use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Redirect, Response},
};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

const USER_ID_KEY: &str = "user_id";
const USERNAME_KEY: &str = "username";
const ADMIN_ID_KEY: &str = "admin_id";
const ADMIN_USERNAME_KEY: &str = "admin_username";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: i32,
    pub username: String,
}

impl AuthUser {
    pub fn new(id: i32, username: String) -> Self {
        Self { id, username }
    }

    pub async fn login(&self, session: &Session) -> Result<(), tower_sessions::session::Error> {
        session.insert(USER_ID_KEY, self.id).await?;
        session.insert(USERNAME_KEY, self.username.clone()).await?;
        Ok(())
    }

    pub async fn logout(session: &Session) -> Result<(), tower_sessions::session::Error> {
        session.remove::<i32>(USER_ID_KEY).await?;
        session.remove::<String>(USERNAME_KEY).await?;
        Ok(())
    }

    pub async fn from_session(session: &Session) -> Option<Self> {
        let id = session.get::<i32>(USER_ID_KEY).await.ok()??;
        let username = session.get::<String>(USERNAME_KEY).await.ok()??;
        Some(Self { id, username })
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

        Self::from_session(&session)
            .await
            .ok_or_else(|| Redirect::to("/login").into_response())
    }
}

// Optional auth user - doesn't redirect if not logged in
#[derive(Debug, Clone)]
pub struct OptionalAuthUser(pub Option<AuthUser>);

#[async_trait]
impl<S> FromRequestParts<S> for OptionalAuthUser
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

        let user = AuthUser::from_session(&session).await;
        Ok(OptionalAuthUser(user))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUser {
    pub id: i32,
    pub username: String,
}

impl AdminUser {
    pub fn new(id: i32, username: String) -> Self {
        Self { id, username }
    }

    pub async fn login(&self, session: &Session) -> Result<(), tower_sessions::session::Error> {
        session.insert(ADMIN_ID_KEY, self.id).await?;
        session
            .insert(ADMIN_USERNAME_KEY, self.username.clone())
            .await?;
        Ok(())
    }

    pub async fn logout(session: &Session) -> Result<(), tower_sessions::session::Error> {
        session.remove::<i32>(ADMIN_ID_KEY).await?;
        session.remove::<String>(ADMIN_USERNAME_KEY).await?;
        Ok(())
    }

    pub async fn from_session(session: &Session) -> Option<Self> {
        let id = session.get::<i32>(ADMIN_ID_KEY).await.ok()??;
        let username = session.get::<String>(ADMIN_USERNAME_KEY).await.ok()??;
        Some(Self { id, username })
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AdminUser
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

        Self::from_session(&session)
            .await
            .ok_or_else(|| Redirect::to("/loginadmin").into_response())
    }
}

// Optional admin user - doesn't redirect if not logged in
#[derive(Debug, Clone)]
pub struct OptionalAdminUser(pub Option<AdminUser>);

#[async_trait]
impl<S> FromRequestParts<S> for OptionalAdminUser
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

        let user = AdminUser::from_session(&session).await;
        Ok(OptionalAdminUser(user))
    }
}
