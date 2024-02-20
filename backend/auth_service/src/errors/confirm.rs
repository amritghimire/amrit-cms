use crate::errors::auth::FetchUserError;
use util_macros::ErrorPayloadMacro;
use utils::errors::{ErrorPayload, ErrorReport};

#[derive(Debug, thiserror::Error, ErrorPayloadMacro)]
pub enum ConfirmUserError {
    #[error("Invalid token provided: {0}")]
    InvalidToken(String),
    #[error("Invalid token format")]
    InvalidTokenUuid(#[source] uuid::Error),
    #[error("Failed to fetch confirmation")]
    ConfirmationDatabaseError(#[source] sqlx::Error),
    #[error("Invalid action type")]
    InvalidActionType,
    #[error("Cannot fetch user")]
    FetchUserFailed(#[source] FetchUserError),
}

impl ErrorReport for ConfirmUserError {
    fn message(&self) -> String {
        self.to_string()
    }
}
