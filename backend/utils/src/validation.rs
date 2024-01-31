use crate::errors::{serde_json_error_response, ErrorPayload};
use async_trait::async_trait;
use axum::extract::{FromRequest, Request};

use axum::extract::rejection::JsonRejection;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::de::DeserializeOwned;

use thiserror::Error;
use validator::Validate;

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedForm<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedForm<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = ServerError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedForm(value))
    }
}

#[derive(Debug, Error)]
pub enum ServerError {
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),

    #[error(transparent)]
    AxumJsonRejection(#[from] JsonRejection),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self {
            ServerError::ValidationError(_) => {
                let message = format!("Input validation error: [{}]", self).replace('\n', ", ");
                (StatusCode::BAD_REQUEST, ErrorPayload::from(message))
            }
            ServerError::AxumJsonRejection(err) => {
                match err {
                    JsonRejection::JsonDataError(err) => {
                        (StatusCode::BAD_REQUEST, ErrorPayload::from(err.to_string()))
                    }
                    JsonRejection::JsonSyntaxError(err) => serde_json_error_response(err),
                    // handle other rejections from the `Json` extractor
                    JsonRejection::MissingJsonContentType(_) => (
                        StatusCode::BAD_REQUEST,
                        ErrorPayload::from(
                            "Missing `Content-Type: application/json` header".to_string(),
                        ),
                    ),
                    JsonRejection::BytesRejection(_) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ErrorPayload::from("Failed to buffer request body".to_string()),
                    ),
                    // we must provide a catch-all case since `JsonRejection` is marked
                    // `#[non_exhaustive]`
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ErrorPayload::from("Unknown error".to_string()),
                    ),
                }
            }
        }
        .into_response()
    }
}
