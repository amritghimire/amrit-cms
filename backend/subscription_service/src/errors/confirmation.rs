use utils::errors::{ErrorPayload, ErrorReport};

#[derive(Debug, thiserror::Error)]
pub enum ConfirmationError {
    #[error("Error occurred when trying to verify token: :{0}")]
    GetSubscriberError(#[source] sqlx::Error),
    #[error("Subscription not found")]
    SubscriptionNotFoundError,
    #[error("Failed to confirm subscription :{0}")]
    ConfirmationFailedError(#[source] sqlx::Error),
}

impl ErrorReport for ConfirmationError {
    fn message(&self) -> String {
        self.to_string()
    }

    fn status(&self) -> u16 {
        match self {
            ConfirmationError::SubscriptionNotFoundError => 401,
            _ => 500,
        }
    }
}


impl From<ConfirmationError> for ErrorPayload {
    fn from(value: ConfirmationError) -> Self {
        ErrorPayload::from_error(value)
    }
}
