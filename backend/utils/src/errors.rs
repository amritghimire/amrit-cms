use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use std::error::Error;

#[derive(serde::Deserialize, Default)]
pub struct ErrorPayload {
    level: String,
    message: String,
    status: u16,
}

impl ErrorPayload {
    pub fn new(message: &str, level: Option<&str>, status: Option<u16>) -> Self {
        Self {
            message: message.to_string(),
            level: level.unwrap_or("error").to_string(),
            status: status.unwrap_or(400),
        }
    }
}

impl From<String> for ErrorPayload {
    fn from(item: String) -> Self {
        ErrorPayload::new(&item, None, None)
    }
}

impl IntoResponse for ErrorPayload {
    fn into_response(self) -> Response {
        let response = json!({
            "message": self.message,
            "level": self.level,
            "status": self.status
        });
        (
            StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(response),
        )
            .into_response()
    }
}

pub fn handle_json_error<T>(
    payload: Result<Json<T>, JsonRejection>,
) -> Result<T, (StatusCode, ErrorPayload)> {
    match payload {
        // if the client sent valid JSON then we're good
        Ok(Json(payload)) => Ok(payload),

        Err(err) => match err {
            JsonRejection::JsonDataError(err) => Err(serde_json_error_response(err)),
            JsonRejection::JsonSyntaxError(err) => Err(serde_json_error_response(err)),
            // handle other rejections from the `Json` extractor
            JsonRejection::MissingJsonContentType(_) => Err((
                StatusCode::BAD_REQUEST,
                ErrorPayload::from("Missing `Content-Type: application/json` header".to_string()),
            )),
            JsonRejection::BytesRejection(_) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorPayload::from("Failed to buffer request body".to_string()),
            )),
            // we must provide a catch-all case since `JsonRejection` is marked
            // `#[non_exhaustive]`
            _ => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorPayload::from("Unknown error".to_string()),
            )),
        },
    }
}

// attempt to extract the inner `serde_json::Error`, if that succeeds we can
// provide a more specific error
pub fn serde_json_error_response<E>(err: E) -> (StatusCode, ErrorPayload)
where
    E: Error + 'static,
{
    if let Some(serde_json_err) = find_error_source::<serde_json::Error>(&err) {
        (
            StatusCode::BAD_REQUEST,
            ErrorPayload::from(format!(
                "Invalid JSON at line {} column {}",
                serde_json_err.line(),
                serde_json_err.column()
            )),
        )
    } else {
        (
            StatusCode::BAD_REQUEST,
            ErrorPayload::from("Unknown error".to_string()),
        )
    }
}

// attempt to downcast `err` into a `T` and if that fails recursively try and
// downcast `err`'s source
fn find_error_source<'a, T>(err: &'a (dyn Error + 'static)) -> Option<&'a T>
where
    T: Error + 'static,
{
    if let Some(err) = err.downcast_ref::<T>() {
        Some(err)
    } else if let Some(source) = err.source() {
        find_error_source(source)
    } else {
        None
    }
}
