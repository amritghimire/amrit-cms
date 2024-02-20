use serde_json::{json, Value};
use util_macros::ErrorPayloadMacro;
use utils::errors::{ErrorPayload, ErrorReport};

#[derive(Debug, thiserror::Error, ErrorPayloadMacro)]
pub enum UserRegistrationError {
    #[error("Failed to acquire a Postgres connection from the pool")]
    Pool(#[source] sqlx::Error),
    #[error("Failed to check username: {0}")]
    UsernameCheck(#[source] UsernameCheckError),
    #[error("Failed to check email: {0}")]
    EmailCheckError(#[source] EmailCheckError),
    #[error("Username not available")]
    UsernameNotAvailable,
    #[error("Email already used")]
    EmailNotAvailable,
    #[error("Insert user failed: {0}")]
    InsertUserFailed(#[source] sqlx::Error),
    #[error("Insert confirmation failed: {0}")]
    InsertConfirmationFailed(#[source] sqlx::Error),
    #[error("Password hash empty")]
    PasswordHashEmpty,
    #[error("Failed to commit transaction: {0}")]
    TransactionCommitError(#[source] sqlx::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum UsernameCheckError {
    #[error("Failed to check for username: {0}")]
    UsernameCheck(#[source] sqlx::Error),
    #[error("Unexpected error occurred")]
    Unexpected,
}

#[derive(Debug, thiserror::Error, ErrorPayloadMacro)]
pub enum FetchUserError {
    #[error("Failed to fetch the username: {0}")]
    UserFetch(#[source] sqlx::Error),
    #[error("Unexpected error occurred")]
    Unexpected,
}

impl ErrorReport for FetchUserError {}

#[derive(Debug, thiserror::Error)]
pub enum EmailCheckError {
    #[error("Failed to check for username: {0}")]
    EmailCheck(#[source] sqlx::Error),
    #[error("Unexpected error occurred")]
    Unexpected,
}

impl ErrorReport for UserRegistrationError {
    fn message(&self) -> String {
        self.to_string()
    }

    fn status(&self) -> u16 {
        match self {
            UserRegistrationError::UsernameCheck(_) => 500,
            UserRegistrationError::Pool(_) => 500,
            UserRegistrationError::UsernameNotAvailable => 400,
            UserRegistrationError::InsertUserFailed(_) => 400,
            UserRegistrationError::PasswordHashEmpty => 500,
            UserRegistrationError::TransactionCommitError(_) => 500,
            UserRegistrationError::EmailCheckError(_) => 400,
            UserRegistrationError::EmailNotAvailable => 400,
            UserRegistrationError::InsertConfirmationFailed(_) => 500,
        }
    }

    fn details(&self) -> Value {
        match self {
            UserRegistrationError::UsernameNotAvailable => ErrorPayload::form_details(
                "username",
                "username_not_available",
                "Username not available",
                None,
            ),
            UserRegistrationError::EmailNotAvailable => ErrorPayload::form_details(
                "email",
                "email_not_available",
                "Email address already used",
                None,
            ),
            _ => json!({}),
        }
    }
}
