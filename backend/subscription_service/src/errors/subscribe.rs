use utils::email::EmailError;
use utils::errors::ErrorPayload;

#[derive(Debug)]
pub enum SubscribeError{
    DatabaseError(sqlx::Error),
    ConfirmationEmailError(EmailError)
}

impl std::fmt::Display for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to create a new subscriber."
        )
    }
}

impl From<sqlx::Error> for SubscribeError {
    fn from(value: sqlx::Error) -> Self {
        tracing::error!("Error occurred {:?}", value);
        Self::DatabaseError(value)
    }
}

impl From<EmailError> for SubscribeError {
    fn from(value: EmailError) -> Self {
        Self::ConfirmationEmailError(value)
    }
}


impl std::error::Error for SubscribeError {}

impl From<SubscribeError> for ErrorPayload {
    fn from(value: SubscribeError) -> Self {
        match value {
            SubscribeError::DatabaseError(err) => {

                if let Some(e) = err.into_database_error() {
                    let message: &str = e.message();
                    if message.contains("subscriptions_email_key") && message.contains("duplicate key value") {
                        tracing::info!("Email already exists");
                        return ErrorPayload::new("Email already subscribed", Some("error"), Some(400));
                    }
                }

                ErrorPayload::new("Unable to add to subscription", Some("error"), Some(500))
            },
            SubscribeError::ConfirmationEmailError(err) => {
                ErrorPayload::new(&err.message, Some("error"), Some(400))
            }
        }
    }
}


