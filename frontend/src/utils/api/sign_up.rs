use crate::entities::user::User;
use dioxus::prelude::FormValue;
use serde::Serialize;
use std::collections::HashMap;

use crate::utils::api::post_request;
use crate::Result;

#[derive(Debug, Serialize, Clone)]
pub struct RegistrationPayload {
    pub username: String,
    pub password: String,
    pub email: String,
    pub confirm_password: String,
    pub name: String,
}

impl From<&HashMap<String, FormValue>> for RegistrationPayload {
    fn from(value: &HashMap<String, FormValue>) -> Self {
        Self {
            username: value["username"].as_value(),
            password: value["password"].as_value(),
            confirm_password: value["confirm_password"].as_value(),
            email: value["email"].as_value(),
            name: value["name"].as_value(),
        }
    }
}

pub async fn signup(payload: RegistrationPayload) -> Result<User> {
    let data = serde_json::to_value(payload)?;
    post_request("/auth/register", &data).await
}
