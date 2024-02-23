use crate::errors::user::UserError;
use crate::extractors::session::UserSession;
use crate::extractors::user::User;
use crate::helpers::user::fetch_user;
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::PgConnection;
use uuid::Uuid;

pub async fn create_new_session(
    transaction: &mut PgConnection,
    user_id: i32,
    details: Value,
) -> Result<String, UserError> {
    let (user_session, token) = UserSession::new(user_id, details);

    sqlx::query!(
        r#"
        INSERT INTO sessions ( identifier, verifier_hash, expiration_date, user_id, extra_info)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        user_session.identifier,
        user_session.verifier_hash,
        user_session.expiration_date,
        user_session.user_id,
        user_session.extra_info
    )
    .execute(transaction)
    .await
    .map_err(UserError::SessionError)?;
    Ok(token)
}

pub async fn user_from_session(
    transaction: &mut PgConnection,
    token: String,
) -> Result<User, UserError> {
    let (identifier, verifier) =
        token
            .split_once('.')
            .ok_or(UserError::AuthorizationTokenInvalid(
                "incomplete token".into(),
            ))?;

    if identifier.is_empty() || verifier.is_empty() {
        Err(UserError::AuthorizationTokenInvalid(
            "empty token part".into(),
        ))?;
    }
    let identifier = Uuid::parse_str(identifier)
        .map_err(|_| UserError::AuthorizationTokenInvalid("invalid token".into()))?;

    let session = sqlx::query_as!(
        UserSession,
        r#"SELECT * from sessions where identifier = $1"#,
        identifier
    )
    .fetch_optional(&mut *transaction)
    .await
    .map_err(UserError::SessionError)?;
    if session.is_none() {
        Err(UserError::AuthorizationTokenInvalid(
            "session not found".into(),
        ))?;
    }
    let mut session = session.unwrap();
    if session.is_expired() {
        delete_session(transaction, session.identifier).await?;
        Err(UserError::AuthorizationTokenInvalid(
            "session expired".into(),
        ))?;
    }
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let verifier_hash = format!("{:x}", hasher.finalize());
    if verifier_hash.ne(&session.verifier_hash) {
        Err(UserError::AuthorizationTokenInvalid(
            "invalid token hash".into(),
        ))?;
    }

    if session.should_extend() {
        sqlx::query!(
            r#"
            UPDATE sessions SET expiration_date = $1 where identifier = $2
            "#,
            session.expiration_date,
            identifier
        )
        .execute(&mut *transaction)
        .await
        .map_err(UserError::SessionError)?;
    }
    let user = fetch_user(&mut *transaction, session.user_id)
        .await
        .map_err(UserError::UserFetchError)?;

    Ok(user)
}

#[tracing::instrument(name = "Deleting session for id", skip(transaction))]
pub async fn delete_session(
    transaction: &mut PgConnection,
    identifier: Uuid,
) -> Result<(), UserError> {
    sqlx::query!(
        r#"
        DELETE FROM sessions where identifier = $1
        "#,
        identifier
    )
    .execute(&mut *transaction)
    .await
    .map_err(UserError::SessionError)?;
    Ok(())
}
