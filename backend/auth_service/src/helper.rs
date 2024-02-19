use crate::errors::auth::{EmailCheckError, UserRegistrationError, UsernameCheckError};
use crate::errors::user::UserError;
use crate::extractor::{Confirmation, ConfirmationActionType, User};
use secrecy::ExposeSecret;
use serde_json::json;
use sqlx::PgConnection;
use utils::email::send_email;
use utils::state::AppState;

#[tracing::instrument(name = "Checking for existing username")]
pub async fn is_username_used(
    transaction: &mut PgConnection,
    username: &str,
) -> Result<bool, UsernameCheckError> {
    sqlx::query!(
        r#"
        SELECT EXISTS(SELECT 1 FROM users WHERE normalized_username = $1)
        "#,
        username
    )
    .fetch_one(transaction)
    .await
    .map_err(UsernameCheckError::UsernameCheck)?
    .exists
    .ok_or(UsernameCheckError::Unexpected)
}

#[tracing::instrument(name = "Checking for existing email")]
pub async fn is_email_used(
    transaction: &mut PgConnection,
    email: &str,
) -> Result<bool, EmailCheckError> {
    sqlx::query!(
        r#"
        SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)
        "#,
        email
    )
    .fetch_one(transaction)
    .await
    .map_err(EmailCheckError::EmailCheck)?
    .exists
    .ok_or(EmailCheckError::Unexpected)
}

#[tracing::instrument(name = "Inserting subscriber to database", skip(transaction, user))]
pub async fn insert_user(
    transaction: &mut PgConnection,
    user: &User,
) -> Result<i32, UserRegistrationError> {
    let password_hash = user.password_hash.clone();
    let output = sqlx::query!(
        r#"
        INSERT INTO users (name, email, username, normalized_username, password_hash, created_at, updated_at,
                                  is_active, is_confirmed)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) returning id;
        "#,
        user.name,
        user.email,
        user.username,
        user.normalized_username,
        password_hash.expose_secret(),
        user.created_at,
        user.updated_at,
        user.is_active,
        user.is_confirmed
    ).fetch_one(transaction).await.map_err(UserRegistrationError::InsertUserFailed)?;
    Ok(output.id)
}

#[tracing::instrument(name = "Send verification link", skip(state, user))]
pub async fn send_verification_link(state: &AppState, user: &User) -> Result<(), UserError> {
    let confirmation = Confirmation::new(
        user.id,
        ConfirmationActionType::UserVerification,
        json!({"email": user.email}),
    );

    let client = state.email_client.to_owned();
    let confirmation_link =
        confirmation.confirmation_url(&state.settings.application.full_url())?;
    let email_content = confirmation.email_contents(&confirmation_link);
    send_email(
        &client,
        user.email.clone(),
        "Please verify your account to proceed".to_string(),
        email_content.0,
        email_content.1,
    )
    .await
    .map_err(UserError::ConfirmationEmailError)?;
    Ok(())
}
