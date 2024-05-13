use crate::errors::auth::UserRegistrationError;
use crate::errors::confirm::ConfirmUserError;
use crate::errors::user::UserError;
use crate::extractors::confirmation::{Confirmation, ConfirmationActionType};
use crate::extractors::user::User;
use email_clients::email::{EmailAddress, EmailObject};
use secrecy::Secret;
use sha2::{Digest, Sha256};
use sqlx::PgConnection;
use tokio::task;
use utils::errors::ErrorPayload;
use utils::state::{AppState, BackgroundTask};
use uuid::Uuid;

#[tracing::instrument(name = "Send verification link", skip(state, user))]
pub async fn send_verification_link(
    state: &AppState,
    user: &User,
    confirmation: &Confirmation,
    token: String,
) -> Result<(), UserError> {
    let confirmation_link =
        confirmation.confirmation_url(&state.settings.application.full_url(), Secret::from(token));
    let email_content = confirmation.email_contents(&confirmation_link);

    let client = state.email_client.to_owned().unwrap();
    let email_object = EmailObject {
        sender: client.get_sender(),
        to: vec![EmailAddress {
            name: user.name.clone(),
            email: user.email.clone(),
        }],
        subject: confirmation.subject(),
        plain: email_content.0,
        html: email_content.1,
    };
    let handle = task::spawn(async move {
        client
            .send_emails(email_object)
            .await
            .map_err(UserError::ConfirmationEmailError)
            .expect("Unable to send confirmation email");
    });
    if let Some(tx) = &state.tasks {
        let _ = tx.send(BackgroundTask::new("send_confirmation_email", handle));
    }

    Ok(())
}

pub async fn add_confirmation(
    transaction: &mut PgConnection,
    confirmation: &Confirmation,
) -> Result<(), UserRegistrationError> {
    sqlx::query!(
        r#"
        INSERT INTO confirmations ( confirmation_id, details, verifier_hash,
         user_id, created_at, expires_at, action_type)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        confirmation.confirmation_id,
        confirmation.details,
        confirmation.verifier_hash,
        confirmation.user_id,
        confirmation.created_at,
        confirmation.expires_at,
        String::from(confirmation.action_type)
    )
    .execute(transaction)
    .await
    .map_err(UserRegistrationError::InsertConfirmationFailed)?;
    Ok(())
}

#[tracing::instrument(name = "Getting confirmation for id", skip(transaction))]
pub async fn get_confirmation(
    transaction: &mut PgConnection,
    confirmation_id: &str,
) -> Result<Confirmation, ConfirmUserError> {
    let id = Uuid::parse_str(confirmation_id).map_err(ConfirmUserError::InvalidTokenUuid)?;
    let confirmation = sqlx::query_as!(
        Confirmation,
        r#"
        SELECT confirmation_id, details, verifier_hash, user_id, created_at, expires_at, action_type from confirmations where confirmation_id = $1
        "#,
        id
    ).fetch_one(&mut *transaction).await.map_err(ConfirmUserError::ConfirmationDatabaseError)?;

    Ok(confirmation)
}

#[tracing::instrument(name = "Getting confirmation for id", skip(transaction))]
pub async fn delete_confirmation(
    transaction: &mut PgConnection,
    confirmation_id: &str,
) -> Result<(), ConfirmUserError> {
    let id = Uuid::parse_str(confirmation_id).map_err(ConfirmUserError::InvalidTokenUuid)?;

    sqlx::query!(
        r#"
        DELETE FROM confirmations where confirmation_id = $1
        "#,
        id
    )
    .execute(&mut *transaction)
    .await
    .map_err(ConfirmUserError::ConfirmationDatabaseError)?;
    Ok(())
}

#[tracing::instrument(name = "Clearing reset confirmations", skip(transaction))]
pub async fn clear_confirmation_action_type(
    transaction: &mut PgConnection,
    user_id: i32,
    action_type: ConfirmationActionType,
) -> Result<(), ConfirmUserError> {
    sqlx::query!(
        r#"
        DELETE FROM confirmations where user_id = $1 and action_type = $2
        "#,
        user_id,
        String::from(action_type)
    )
    .execute(&mut *transaction)
    .await
    .map_err(ConfirmUserError::ConfirmationDatabaseError)?;
    Ok(())
}

#[tracing::instrument(name = "Marking user as confirmed.", skip(transaction))]
pub async fn mark_user_as_confirmed(
    transaction: &mut PgConnection,
    user_id: i32,
) -> Result<(), ConfirmUserError> {
    sqlx::query!(
        "update users set is_confirmed = true where id = $1;",
        user_id
    )
    .execute(&mut *transaction)
    .await
    .map_err(ConfirmUserError::ConfirmationDatabaseError)?;
    sqlx::query!(
        "delete from confirmations where user_id = $1 and action_type = $2;",
        user_id,
        String::from(ConfirmationActionType::UserVerification)
    )
    .execute(&mut *transaction)
    .await
    .map_err(ConfirmUserError::ConfirmationDatabaseError)?;
    Ok(())
}

pub async fn check_confirmation(
    token: String,
    transaction: &mut PgConnection,
) -> Result<Confirmation, ErrorPayload> {
    let (confirmation_id, verifier) = token
        .split_once('.')
        .ok_or(ConfirmUserError::InvalidToken("incomplete token".into()))?;
    if confirmation_id.is_empty() || verifier.is_empty() {
        Err(ConfirmUserError::InvalidToken("empty token part".into()))?;
    }
    let confirmation = get_confirmation(transaction, confirmation_id).await?;
    if confirmation.is_expired() {
        delete_confirmation(transaction, confirmation_id).await?;
        Err(ConfirmUserError::InvalidToken("expired token".into()))?;
    }

    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    // Read hash digest and consume hasher
    let verifier_hash = format!("{:x}", hasher.finalize());
    if verifier_hash.ne(&confirmation.verifier_hash) {
        Err(ConfirmUserError::InvalidToken("invalid hash".into()))?;
    }
    Ok(confirmation)
}
