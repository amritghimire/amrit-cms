use sqlx::Error;
use utils::errors::ErrorPayload;

#[derive(Debug)]
pub enum ConfirmationError {
    GetSubscriberError(GetSubscriberError),
    SubscriptionNotFoundError(SubscriptionNotFoundError),
    ConfirmationFailedError(ConfirmationFailedError)
}

#[derive(Debug)]
pub struct GetSubscriberError {
    err: Error
}

#[derive(Debug)]
pub struct SubscriptionNotFoundError {}

#[derive(Debug)]
pub struct ConfirmationFailedError {
    err: Error
}

impl std::fmt::Display for ConfirmationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to confirm subscription."
        )
    }
}


impl std::fmt::Display for GetSubscriberError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error occurred when trying to verify token"
        )
    }
}


impl std::fmt::Display for SubscriptionNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cannot find the token"
        )
    }
}


impl std::fmt::Display for ConfirmationFailedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cannot confirm the subscription"
        )
    }
}


impl std::error::Error for ConfirmationError {}
impl std::error::Error for GetSubscriberError {}
impl std::error::Error for SubscriptionNotFoundError {}
impl std::error::Error for ConfirmationFailedError {}

impl From<GetSubscriberError> for ConfirmationError {
    fn from(error: GetSubscriberError) -> Self {
        ConfirmationError::GetSubscriberError(error)
    }
}

impl From<SubscriptionNotFoundError> for ConfirmationError {
    fn from(error: SubscriptionNotFoundError) -> Self {
        ConfirmationError::SubscriptionNotFoundError(error)
    }
}

impl From<ConfirmationFailedError> for ConfirmationError {
    fn from(error: ConfirmationFailedError) -> Self {
        ConfirmationError::ConfirmationFailedError(error)
    }
}

impl From<Error> for GetSubscriberError {
    fn from(err: Error) -> Self {
        tracing::error!("Error occurred {:?}", err);

        Self {
            err
        }
    }
}


impl From<Error> for ConfirmationFailedError {
    fn from(err: Error) -> Self {
        tracing::error!("Error occurred {:?}", err);

        Self {
            err
        }
    }
}

impl From<ConfirmationError> for ErrorPayload {
    fn from(value: ConfirmationError) -> Self {
        match value {
            ConfirmationError::SubscriptionNotFoundError(err) => {
                ErrorPayload::new(&err.to_string(), Some("error"), Some(401))
            }
            ConfirmationError::GetSubscriberError(err) => {
                ErrorPayload::new(&err.to_string(), Some("error"), Some(500))
            }
            ConfirmationError::ConfirmationFailedError(err) => {
                ErrorPayload::new(&err.to_string(), Some("error"), Some(500))
            }
        }
    }
}