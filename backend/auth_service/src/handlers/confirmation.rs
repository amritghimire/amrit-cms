use crate::errors::auth::UserRegistrationError;
use crate::errors::confirm::ConfirmUserError;
use crate::errors::confirm::ConfirmUserError::UserAlreadyVerified;
use crate::extractors::authentication::LoggedInUser;
use crate::extractors::confirmation::{Confirmation, ConfirmationActionType};
use crate::extractors::user::User;
use crate::helpers::confirmation;
use crate::helpers::confirmation::{
    add_confirmation, delete_confirmation, mark_user_as_confirmed, send_verification_link,
};
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use sqlx::PgConnection;
use utils::errors::ErrorPayload;
use utils::state::AppState;

pub async fn confirm(
    State(state): State<AppState>,
    Path(token): Path<String>,
    user: LoggedInUser,
) -> Result<impl IntoResponse, ErrorPayload> {
    let pool = &state.connection;
    let LoggedInUser { user, .. } = user;

    tracing::info!("starting confirmation token");
    let mut transaction = pool.begin().await.map_err(UserRegistrationError::Pool)?;

    let confirmation = confirmation::check_confirmation(token, &mut transaction).await?;

    if user.id != confirmation.user_id {
        Err(ConfirmUserError::InsufficientPermission(
            "login with the user you want to verify".into(),
        ))?;
    }

    let response = match confirmation.action_type {
        ConfirmationActionType::UserVerification => {
            Ok(verify_user(&mut transaction, &confirmation, user).await?)
        }
        ConfirmationActionType::Invalid => {
            Err(ConfirmUserError::InvalidToken("invalid token type".into()))?
        }
        ConfirmationActionType::PasswordReset => Err(ConfirmUserError::InvalidToken(
            "password reset not supported here".into(),
        ))?,
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
    user: User,
) -> Result<impl IntoResponse, ConfirmUserError> {
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

pub async fn resend_verification(
    State(state): State<AppState>,
    user: LoggedInUser,
) -> Result<impl IntoResponse, ErrorPayload> {
    let pool = &state.connection;
    let LoggedInUser { user, .. } = user;
    if user.is_confirmed {
        return Err(UserAlreadyVerified.into());
    }

    tracing::info!("starting new verification token");
    let mut transaction = pool.begin().await.map_err(UserRegistrationError::Pool)?;
    let (confirmation, confirmation_token) = Confirmation::new(
        user.id,
        ConfirmationActionType::UserVerification,
        json!({"email": user.email}),
    );
    add_confirmation(&mut transaction, &confirmation).await?;
    send_verification_link(&state, &user, &confirmation, confirmation_token).await?;
    transaction
        .commit()
        .await
        .map_err(UserRegistrationError::TransactionCommitError)?;
    Ok(Json(json!({})))
}
