use utils::email::EmailError;
use utils::errors::{ErrorPayload, ErrorReport};

#[derive(Debug, thiserror::Error)]
pub enum SubscribeError {
    #[error("Failed to acquire a Postgres connection from the pool")]
    PoolError(#[source] sqlx::Error),
    #[error("Failed to add subscriber: {0}")]
    InsertSubscribeError(#[source] sqlx::Error),
    #[error("Failed to store token: {0}")]
    StoreTokenError(#[source] sqlx::Error),
    #[error("Failed to send confirmation email: {0}")]
    ConfirmationEmailError(#[source] EmailError),
    #[error("Failed to commit transaction: {0}")]
    TransactionCommitError(#[source] sqlx::Error),
}

impl ErrorReport for SubscribeError {
    fn message(&self) -> String {
        match self {
            SubscribeError::PoolError(e) => e.to_string(),
            SubscribeError::TransactionCommitError(e) => e.to_string(),
            SubscribeError::InsertSubscribeError(e) => e.to_string(),
            SubscribeError::StoreTokenError(e) => e.to_string(),
            SubscribeError::ConfirmationEmailError(e) => e.to_string(),
        }
    }

    fn status(&self) -> u16 {
        match self {
            SubscribeError::PoolError(_) => 500,
            SubscribeError::TransactionCommitError(_) => 500,
            SubscribeError::InsertSubscribeError(_) => 500,
            SubscribeError::StoreTokenError(_) => 500,
            SubscribeError::ConfirmationEmailError(_) => 500,
        }
    }
}

impl From<SubscribeError> for ErrorPayload {
    fn from(value: SubscribeError) -> Self {
        ErrorPayload::from_error(value)
    }
}
