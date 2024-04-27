use crate::errors::auth::UserRegistrationError;
use crate::errors::confirm::ConfirmUserError;
use crate::errors::user::UserError;
use crate::extractors::confirmation::Confirmation;
use crate::extractors::user::User;
use email_clients::email::{EmailAddress, EmailObject};
use secrecy::Secret;
use sqlx::PgConnection;
use utils::state::AppState;
use uuid::Uuid;

#[tracing::instrument(name = "Send verification link", skip(state, user))]
pub async fn send_verification_link(
    state: &AppState,
    user: &User,
    confirmation: &Confirmation,
    token: String,
) -> Result<(), UserError> {
    let client = state.email_client.to_owned().unwrap();
    let confirmation_link =
        confirmation.confirmation_url(&state.settings.application.full_url(), Secret::from(token));
    let email_content = confirmation.email_contents(&confirmation_link);

    let email_object = EmailObject {
        sender: client.get_sender().to_string(),
        to: vec![EmailAddress {
            name: user.name.clone(),
            email: user.email.clone(),
        }],
        subject: "Please verify your account to proceed".to_string(),
        plain: email_content.0,
        html: email_content.1,
    };
    client
        .send_emails(email_object)
        .await
        .map_err(UserError::ConfirmationEmailError)?;
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
    sqlx::query!("delete from confirmations where user_id = $1;", user_id)
        .execute(&mut *transaction)
        .await
        .map_err(ConfirmUserError::ConfirmationDatabaseError)?;
    Ok(())
}
