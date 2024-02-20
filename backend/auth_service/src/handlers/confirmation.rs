use crate::errors::auth::UserRegistrationError;
use crate::errors::confirm::ConfirmUserError;
use crate::extractor::{Confirmation, ConfirmationActionType};
use crate::helpers::confirmation::{delete_confirmation, get_confirmation, mark_user_as_confirmed};
use crate::helpers::user::fetch_user;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::PgConnection;
use utils::errors::ErrorPayload;
use utils::state::AppState;

pub async fn confirm(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<impl IntoResponse, ErrorPayload> {
    let pool = &state.connection;

    tracing::info!("starting confirmation token");
    let mut transaction = pool.begin().await.map_err(UserRegistrationError::Pool)?;

    let (confirmation_id, verifier) = token
        .split_once('.')
        .ok_or(ConfirmUserError::InvalidToken("incomplete token".into()))?;
    if confirmation_id.is_empty() || verifier.is_empty() {
        Err(ConfirmUserError::InvalidToken("empty token part".into()))?;
    }
    let confirmation = get_confirmation(&mut transaction, confirmation_id).await?;
    if confirmation.is_expired() {
        delete_confirmation(&mut transaction, confirmation_id).await?;
        Err(ConfirmUserError::InvalidToken("expired token".into()))?;
    }

    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    // Read hash digest and consume hasher
    let verifier_hash = format!("{:x}", hasher.finalize());
    if verifier_hash.ne(&confirmation.verifier_hash) {
        Err(ConfirmUserError::InvalidToken("invalid hash".into()))?;
    }
    let response = match confirmation.action_type {
        ConfirmationActionType::UserVerification => {
            Ok(verify_user(&mut transaction, &confirmation).await?)
        }
        ConfirmationActionType::Invalid => {
            Err(ConfirmUserError::InvalidToken("invalid token type".into()))?
        }
    };
    transaction
        .commit()
        .await
        .map_err(UserRegistrationError::TransactionCommitError)?;
    response
}

async fn verify_user(
    transaction: &mut PgConnection,
    confirmation: &Confirmation,
) -> Result<impl IntoResponse, ConfirmUserError> {
    let user = fetch_user(transaction, confirmation.user_id)
        .await
        .map_err(ConfirmUserError::FetchUserFailed)?;
    let confirmation_email = confirmation
        .details
        .clone()
        .ok_or(ConfirmUserError::InvalidToken(
            "missing confirmation details".into(),
        ))?;
    let confirmation_email =
        confirmation_email
            .get("email")
            .ok_or(ConfirmUserError::InvalidToken(
                "missing confirmation email".into(),
            ))?;
    let confirmation_email = confirmation_email
        .as_str()
        .ok_or(ConfirmUserError::InvalidToken("invalid email set".into()))?;
    if user.email.ne(&confirmation_email) {
        delete_confirmation(transaction, &confirmation.confirmation_id.to_string()).await?;
        Err(ConfirmUserError::InvalidToken("email mismatch".into()))?;
    }
    mark_user_as_confirmed(transaction, user.id).await?;
    Ok(Json(json!({})))
}
