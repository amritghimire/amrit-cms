use serde_json::{json, Value};
use util_macros::ErrorPayloadMacro;
use utils::email::EmailError;
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
    #[error("Confirmation token missing")]
    ConfirmationTokenMissing,
    #[error("Failed to send confirmation email")]
    ConfirmationEmailError(#[source] EmailError),
}

impl ErrorReport for UserError {
    fn message(&self) -> String {
        self.to_string()
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
            _ => {
                json!({})
            }
        }
    }
}
