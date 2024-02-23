use crate::errors::auth::UserRegistrationError;
use crate::errors::user::UserError;
use crate::extractors::session::SESSION_TOKEN_COOKIE;
use crate::extractors::user::User;
use crate::helpers::sessions::user_from_session;
use axum::http::header::AUTHORIZATION;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, HeaderMap},
    response::{IntoResponse, Response},
};
use axum_extra::extract::{cookie::Key, SignedCookieJar};

use utils::errors::ErrorPayload;
use utils::state::AppState;

pub struct LoggedInUser {
    pub user: User,
}

pub struct AuthenticatedUser {
    pub user: User,
}

pub struct AuthenticationHeaderUser {
    pub user: User,
}

impl From<User> for LoggedInUser {
    fn from(user: User) -> Self {
        Self { user }
    }
}

impl From<User> for AuthenticationHeaderUser {
    fn from(user: User) -> Self {
        Self { user }
    }
}

impl TryFrom<User> for AuthenticatedUser {
    type Error = UserError;

    fn try_from(user: User) -> Result<Self, Self::Error> {
        if user.is_confirmed {
            return Ok(Self { user });
        }
        Err(UserError::UserNotVerified)
    }
}

#[async_trait]
impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = process_session_from_parts(parts, state, true).await;
        if let Err(err) = user {
            return Err(err.into_response());
        }
        match AuthenticatedUser::try_from(user.unwrap()) {
            Ok(auth_user) => Ok(auth_user),
            Err(err) => Err(ErrorPayload::from_error(err).into_response()),
        }
    }
}

#[async_trait]
impl FromRequestParts<AppState> for LoggedInUser {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = process_session_from_parts(parts, state, true).await;
        match user {
            Ok(user) => Ok(user.into()),
            Err(err) => Err(err.into_response()),
        }
    }
}

#[async_trait]
impl FromRequestParts<AppState> for AuthenticationHeaderUser {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = process_session_from_parts(parts, state, false).await;
        match user {
            Ok(user) => Ok(user.into()),
            Err(err) => Err(err.into_response()),
        }
    }
}

async fn process_session_from_parts(
    parts: &mut Parts,
    state: &AppState,
    cookie: bool,
) -> Result<User, ErrorPayload> {
    let headers = HeaderMap::from_request_parts(parts, state).await?;
    let mut token = "".to_string();
    if let Some(header) = headers.get(AUTHORIZATION) {
        if let Ok(header) = header.to_str() {
            token = header.to_string();
        }
    }

    if cookie && token.is_empty() {
        let jar = SignedCookieJar::<Key>::from_request_parts(parts, state).await?;
        let session_token = jar
            .get(SESSION_TOKEN_COOKIE)
            .map(|cookie| cookie.value().to_owned());
        if let Some(session_token) = session_token {
            token = session_token;
        }
    }

    tracing::debug!("Token found {:?}", token);

    if token.is_empty() {
        Err(UserError::AuthorizationTokenInvalid(
            "token not available".into(),
        ))?;
    }

    let pool = &state.connection;
    let mut transaction = pool.begin().await.map_err(UserRegistrationError::Pool)?;
    let user = user_from_session(&mut transaction, token).await?;
    Ok(user)
}
