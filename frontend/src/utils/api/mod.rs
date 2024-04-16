use crate::errors::{ApplicationError, ErrorPayload};
use crate::Result;
use reqwest::{Response, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::Value;

pub(crate) mod confirm;
pub(crate) mod logout;
pub(crate) mod me;
pub(crate) mod sign_in;
pub(crate) mod sign_up;

pub fn form_url(path: &str) -> String {
    let base_url = web_sys::window().unwrap().location().origin().unwrap();
    format!("{}/api{}", base_url, path)
}

pub async fn post_request<T: DeserializeOwned>(path: &str, data: &Value) -> Result<T> {
    let url = form_url(path);
    let client = reqwest::Client::new();
    let response = client.post(url).json(data).send().await?;
    let value = process_response(response).await?;
    Ok(value)
}

pub async fn process_response<T: DeserializeOwned>(response: Response) -> Result<T> {
    if response.status() == StatusCode::UNAUTHORIZED {
        Err(ApplicationError::Unauthorized)?
    }
    if response.status() == StatusCode::FORBIDDEN {
        Err(ApplicationError::Forbidden)?
    }
    if response.status() == StatusCode::BAD_REQUEST {
        let value: ErrorPayload = response.json().await?;
        return Err(ApplicationError::BadRequestError(value));
    }
    if response.status().is_client_error() {
        Err(ApplicationError::RequestAPIFailed)?
    }
    if response.status().is_server_error() {
        Err(ApplicationError::RequestServerError)?
    }
    let value = response.json().await;
    match value {
        Ok(value) => Ok(value),
        Err(e) => Err(ApplicationError::SerializeFailed(e)),
    }
}
