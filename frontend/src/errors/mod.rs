use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("You dont have enough permission to make this API Call")]
    Forbidden,
    #[error("Failed to make an API Call")]
    RequestAPIFailed,
    #[error("Server not responding properly")]
    RequestServerError,
    #[error("Failed to make an API Call: {0} ")]
    RequestError(#[from] reqwest::Error),
    #[error("Failed to serialize API response: {0}")]
    SerializeFailed(reqwest::Error),
    #[error("Failed to serialize API request: {0}")]
    SerializeError(#[from] serde_json::Error),
    #[error("Bad request ")]
    BadRequestError(ErrorPayload),
}

#[derive(serde::Deserialize, serde::Serialize, Default, Debug, Clone, PartialEq)]
pub struct ErrorPayload {
    pub level: String,
    pub message: String,
    pub status: u16,
    pub details: Value,
}

impl ErrorPayload {
    pub fn error_for_field(&self, field: &str) -> String {
        if let Some(detail) = self.details.get(field.to_string()) {
            if detail.is_array() {
                let mut message = "".to_string();
                for d in detail.as_array().unwrap() {
                    if let Some(m) = d.get("message") {
                        message.push_str(m.as_str().unwrap_or(""))
                    }
                }
                return message;
            } else if detail.is_object() {
                if let Some(m) = detail.get("message") {
                    return m.as_str().unwrap_or("").to_string();
                }
            }
        }
        "".to_string()
    }
}
