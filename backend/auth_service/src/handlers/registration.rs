use crate::errors::auth::UserRegistrationError;
use crate::extractor::User;
use crate::helper::{insert_user, is_email_used, is_username_used, send_verification_link};
use crate::payload::RegisterPayload;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use utils::errors::ErrorPayload;
use utils::state::AppState;
use utils::validation::ValidatedForm;

#[tracing::instrument(name="Starting a registration",
skip(state, payload), fields(
name= %payload.name,
username= %payload.username,
email= %payload.email
)
)]
pub async fn register(
    State(state): State<AppState>,
    ValidatedForm(payload): ValidatedForm<RegisterPayload>,
) -> Result<impl IntoResponse, ErrorPayload> {
    let pool = &state.connection;

    tracing::info!("Creating a new registration");
    let mut transaction = pool.begin().await.map_err(UserRegistrationError::Pool)?;

    let normalized_username = User::normalize_username(&payload.username)?;
    User::check_acceptable_password(&payload.password, &[&payload.name, &payload.username])?;
    let username_used = is_username_used(&mut transaction, &normalized_username)
        .await
        .map_err(UserRegistrationError::UsernameCheck)?;
    if username_used {
        return Err(UserRegistrationError::UsernameNotAvailable.into());
    }

    if is_email_used(&mut transaction, &payload.email)
        .await
        .map_err(UserRegistrationError::EmailCheckError)?
    {
        return Err(UserRegistrationError::EmailNotAvailable.into());
    }

    let mut user = User::try_from(payload)?;
    let id = insert_user(&mut transaction, &user).await?;
    user.id = id;

    transaction
        .commit()
        .await
        .map_err(UserRegistrationError::TransactionCommitError)?;

    send_verification_link(&state, &user).await?;
    Ok(Json(user))
}
