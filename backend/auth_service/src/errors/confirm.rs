use crate::errors::auth::FetchUserError;
use util_macros::ErrorPayloadMacro;
use utils::errors::{ErrorPayload, ErrorReport};

#[derive(Debug, thiserror::Error, ErrorPayloadMacro)]
pub enum ConfirmUserError {
    #[error("Invalid token provided: {0}")]
    InvalidToken(String),
    #[error("insufficient permission: {0}")]
    InsufficientPermission(String),
    #[error("Invalid token format")]
    InvalidTokenUuid(#[source] uuid::Error),
    #[error("Failed to fetch confirmation")]
    ConfirmationDatabaseError(#[source] sqlx::Error),
    #[error("Invalid action type")]
    InvalidActionType,
    #[error("Cannot fetch user")]
    FetchUserFailed(#[source] FetchUserError),
    #[error("User already verified")]
    UserAlreadyVerified,
}

impl ErrorReport for ConfirmUserError {
    fn message(&self) -> String {
        self.to_string()
    }
    fn status(&self) -> u16 {
        match self {
            ConfirmUserError::InsufficientPermission(_) => 401,
            _ => 400,
        }
    }
}
