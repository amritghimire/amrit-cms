use crate::errors::auth::FetchUserError;
use email_clients::errors::EmailError;
use serde_json::{json, Value};
use util_macros::ErrorPayloadMacro;
use utils::errors::{ErrorPayload, ErrorReport};
use zxcvbn::ZxcvbnError;

#[derive(Debug, thiserror::Error, ErrorPayloadMacro)]
pub enum UserError {
    #[error("invalid password provided: {0}")]
    PasswordHashError(String),
    #[error("username validation failed: {0}")]
    NormalizeUserFailed(String),
    #[error("failed to check password strength")]
    PasswordCheckFailed(#[source] ZxcvbnError),
    #[error("Weak password")]
    WeakPassword,
    #[error("Failed to send confirmation email: {0}")]
    ConfirmationEmailError(#[source] EmailError),
    #[error("User not verified")]
    UserNotVerified,
    #[error("Authorization token invalid: {0}")]
    AuthorizationTokenInvalid(String),
    #[error("Session database failed")]
    SessionError(#[source] sqlx::Error),
    #[error("user fetch error")]
    UserFetchError(#[source] FetchUserError),
}

impl ErrorReport for UserError {
    fn message(&self) -> String {
        self.to_string()
    }

    fn status(&self) -> u16 {
        match self {
            UserError::UserNotVerified => 403,
            UserError::ConfirmationEmailError(_) => 500,
            UserError::PasswordCheckFailed(_) => 500,
            UserError::AuthorizationTokenInvalid(_) => 401,
            _ => 400,
        }
    }

    fn details(&self) -> Value {
        match self {
            UserError::PasswordHashError(e) => {
                ErrorPayload::form_details("password", "password_hash_error", e, None)
            }
            UserError::NormalizeUserFailed(e) => {
                ErrorPayload::form_details("username", "invalid_username", e, None)
            }
            UserError::PasswordCheckFailed(_) => ErrorPayload::form_details(
                "password",
                "password_check_failed",
                "Failed to check password strength",
                None,
            ),
            UserError::WeakPassword => {
                ErrorPayload::form_details("password", "weak_password", "Weak password", None)
            }
            UserError::UserNotVerified => {
                ErrorPayload::form_details("auth", "user_not_verified", "User not verified", None)
            }
            _ => {
                json!({})
            }
        }
    }
}
