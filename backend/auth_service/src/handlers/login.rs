use crate::errors::auth::{UserLoginError, UserRegistrationError};
use crate::extractors::session::SESSION_TOKEN_COOKIE;

use crate::helpers::sessions::create_new_session;
use crate::helpers::user::fetch_by_username;
use axum::extract::State;
use axum::http::header::AUTHORIZATION;
use axum::http::HeaderValue;
use axum::response::IntoResponse;
use axum::Json;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::SignedCookieJar;
use serde::Deserialize;
use serde_json::json;
use validator::Validate;

use crate::extractors::confirmation::ConfirmationActionType;
use crate::helpers::confirmation::clear_confirmation_action_type;
use utils::errors::ErrorPayload;
use utils::state::AppState;
use utils::validation::ValidatedForm;

#[derive(Debug, Deserialize, Validate)]
pub struct LoginForm {
    #[validate(
        length(min = 3, message = "Username cannot be empty"),
        non_control_character
    )]
    pub username: String,
    #[validate(length(min = 8, max = 72, message = "Password must contains 8-72 characters"))]
    pub password: String,
}

#[tracing::instrument(name = "Starting a login",
skip(state, payload), fields(
username = % payload.username,
)
)]
pub async fn login(
    jar: SignedCookieJar,
    State(state): State<AppState>,
    ValidatedForm(payload): ValidatedForm<LoginForm>,
) -> Result<impl IntoResponse, ErrorPayload> {
    let pool = &state.connection;

    let mut transaction = pool.begin().await.map_err(UserLoginError::Pool)?;

    let user = fetch_by_username(&mut transaction, &payload.username)
        .await
        .map_err(UserLoginError::UnexpectedError)?;
    let user = user.ok_or(UserLoginError::LoginFailed("username not found".into()))?;

    if user.check_password(&payload.password) {
        let session_token = create_new_session(&mut transaction, user.id, json!({}))
            .await
            .map_err(UserLoginError::UnexpectedUserError)?;

        clear_confirmation_action_type(
            &mut transaction,
            user.id,
            ConfirmationActionType::PasswordReset,
        )
        .await?;

        transaction
            .commit()
            .await
            .map_err(UserLoginError::DatabaseError)?;

        let session_header =
            HeaderValue::from_str(&session_token).map_err(UserRegistrationError::HeaderError)?;

        let jar = jar.add(Cookie::new(SESSION_TOKEN_COOKIE, session_token));

        let mut response = (jar, Json(user)).into_response();
        response.headers_mut().insert(AUTHORIZATION, session_header);
        Ok(response)
    } else {
        Err(UserLoginError::LoginFailed("username or password is incorrect".into()).into())
    }
}
