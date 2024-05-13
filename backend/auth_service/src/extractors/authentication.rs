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
use uuid::Uuid;

use utils::errors::ErrorPayload;
use utils::state::AppState;

// Simple login.
pub struct LoggedInUser {
    pub session: Uuid,
    pub user: User,
}

// This checks for verification as well.
pub struct AuthenticatedUser {
    pub session: Uuid,
    pub user: User,
}

// This checks for header only.
pub struct AuthenticationHeaderUser {
    pub session: Uuid,
    pub user: User,
}

impl AuthenticatedUser {
    pub fn new(user: User, session: Uuid) -> Result<Self, UserError> {
        if user.is_confirmed {
            return Ok(Self { user, session });
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
        let result = process_session_from_parts(parts, state, true).await;
        if let Err(err) = result {
            return Err(err.into_response());
        }
        let (user, session) = result.unwrap();
        match AuthenticatedUser::new(user, session) {
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
            Ok((user, session)) => Ok(LoggedInUser { session, user }),
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
        let result = process_session_from_parts(parts, state, false).await;
        match result {
            Ok((user, session)) => Ok(AuthenticationHeaderUser { session, user }),
            Err(err) => Err(err.into_response()),
        }
    }
}

async fn process_session_from_parts(
    parts: &mut Parts,
    state: &AppState,
    cookie: bool,
) -> Result<(User, Uuid), ErrorPayload> {
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
    let result = user_from_session(&mut transaction, token).await?;
    Ok(result)
}
