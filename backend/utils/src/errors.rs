use axum::extract::rejection::{ExtensionRejection, JsonRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use phf::phf_map;
use serde_json::{json, Value};
use std::convert::Infallible;
use std::error::Error;

static DATABASE_ERRORS: phf::Map<&'static str, (&'static str, u16)> = phf_map! {
    "duplicate key value violates unique constraint \"subscriptions_email_key\"" => ("Email already subscribed", 400),
    "duplicate key value violates unique constraint \"users_email_key\"" => ("Email already registered", 400),
    "duplicate key value violates unique constraint \"users_username_key\"" => ("Username not available", 400),
    "duplicate key value violates unique constraint \"users_normalized_username_key\"" => ("Username not available", 400),
};

pub trait ErrorReport {
    fn level(&self) -> &'static str {
        "error"
    }

    fn message(&self) -> String {
        "Unexpected error occurred".to_string()
    }

    fn status(&self) -> u16 {
        400
    }

    fn details(&self) -> serde_json::Value {
        json!({})
    }
}

#[derive(serde::Deserialize, Default, Debug)]
pub struct ErrorPayload {
    level: String,
    message: String,
    status: u16,
    details: Value,
}

impl ErrorPayload {
    pub fn new(message: &str, level: Option<&str>, status: Option<u16>) -> Self {
        Self {
            message: message.to_string(),
            level: level.unwrap_or("error").to_string(),
            status: status.unwrap_or(400),
            details: json!({}),
        }
    }

    pub fn from_error(error: impl ErrorReport) -> Self {
        let mut message = error.message();
        let mut status = error.status();
        for (key, value) in DATABASE_ERRORS.into_iter() {
            if message.contains(key) {
                message = value.0.to_string();
                status = value.1;
                break;
            }
        }
        if (400..499).contains(&status) {
            tracing::info!("{} ({}) {} ", error.level(), status, &message);
        } else if status > 500 {
            tracing::error!("{} ({}) {} ", error.level(), status, &message);
        }

        Self {
            message: message.to_string(),
            level: error.level().to_string(),
            status,
            details: error.details(),
        }
    }

    pub fn set_details(&mut self, value: Value) {
        self.details = value;
    }

    pub fn form_details(key: &str, code: &str, message: &str, value: Option<&str>) -> Value {
        let mut field_value = json!(
            {
                "code": code,
                "message": message,
                "params": {}
            }
        );
        if let Some(value) = value {
            field_value["params"] = json!({
                "value": value
            });
        }
        json!({
            key: [field_value]
        })
    }
}

impl std::fmt::Display for ErrorPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({} error of {} level) {}",
            self.message, self.status, self.level, self.details
        )
    }
}

impl Error for ErrorPayload {}

impl From<String> for ErrorPayload {
    fn from(item: String) -> Self {
        ErrorPayload::new(&item, None, None)
    }
}

impl From<ExtensionRejection> for ErrorPayload {
    fn from(value: ExtensionRejection) -> Self {
        ErrorPayload::new(&value.to_string(), None, None)
    }
}

impl From<Infallible> for ErrorPayload {
    fn from(value: Infallible) -> Self {
        ErrorPayload::new(&value.to_string(), None, None)
    }
}

impl IntoResponse for ErrorPayload {
    fn into_response(self) -> Response {
        let response = json!({
            "message": self.message,
            "level": self.level,
            "status": self.status,
            "details": self.details
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
        // if the frontend sent valid JSON then we're good
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
