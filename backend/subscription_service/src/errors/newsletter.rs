use util_macros::ErrorPayloadMacro;
use utils::errors::{ErrorPayload, ErrorReport};

#[derive(Debug, thiserror::Error, ErrorPayloadMacro)]
pub enum NewsletterError {
    #[error("Failed to acquire a Postgres connection from the pool")]
    PoolError(#[source] sqlx::Error),
    #[error("Failed to add subscriber: {0}")]
    ConfirmedSubscribersError(#[source] sqlx::Error),
}

impl ErrorReport for NewsletterError {
    fn message(&self) -> String {
        self.to_string()
    }

    fn status(&self) -> u16 {
        500
    }
}
