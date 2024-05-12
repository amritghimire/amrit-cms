use crate::errors::auth::UserRegistrationError;
use crate::extractors::confirmation::{Confirmation, ConfirmationActionType};
use crate::extractors::session::SESSION_TOKEN_COOKIE;
use crate::extractors::user::User;
use crate::helpers::confirmation::{add_confirmation, send_verification_link};
use crate::helpers::sessions::create_new_session;
use crate::helpers::user::insert_user;
use crate::payload::RegisterPayload;
use axum::extract::State;
use axum::http::header::AUTHORIZATION;
use axum::http::HeaderValue;
use axum::response::IntoResponse;
use axum::Json;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::SignedCookieJar;
use serde_json::json;
use utils::errors::ErrorPayload;
use utils::state::AppState;
use utils::validation::ValidatedForm;

#[tracing::instrument(name = "Starting a registration",
skip(state, payload), fields(
name = % payload.name,
username = % payload.username,
email = % payload.email
)
)]
pub async fn register(
    jar: SignedCookieJar,
    State(state): State<AppState>,
    ValidatedForm(payload): ValidatedForm<RegisterPayload>,
) -> Result<impl IntoResponse, ErrorPayload> {
    let pool = &state.connection;

    let mut transaction = pool.begin().await.map_err(UserRegistrationError::Pool)?;

    User::check_acceptable_password(&payload.password, &[&payload.name, &payload.username])?;
    let mut user = User::try_from(payload)?;
    let id = insert_user(&mut transaction, &user).await?;
    user.id = id;
    let (confirmation, confirmation_token) = Confirmation::new(
        user.id,
        ConfirmationActionType::UserVerification,
        json!({"email": user.email}),
    );
    add_confirmation(&mut transaction, &confirmation).await?;
    let session_token = create_new_session(&mut transaction, user.id, json!({})).await?;

    transaction
        .commit()
        .await
        .map_err(UserRegistrationError::TransactionCommitError)?;

    send_verification_link(&state, &user, &confirmation, confirmation_token).await?;
    let session_header =
        HeaderValue::from_str(&session_token).map_err(UserRegistrationError::HeaderError)?;

    let jar = jar.add(Cookie::new(SESSION_TOKEN_COOKIE, session_token));
    let mut response = (jar, Json(user)).into_response();
    response.headers_mut().insert(AUTHORIZATION, session_header);
    Ok(response)
}
