use crate::entities::input::UserInput;
use crate::entities::user::User;
use dioxus::prelude::{Readable, Signal};
use serde::Serialize;

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

impl From<Signal<UserInput>> for RegistrationPayload {
    fn from(signal: Signal<UserInput>) -> Self {
        let value = signal.read();
        Self {
            username: value.get("username"),
            password: value.get("password"),
            confirm_password: value.get("confirm_password"),
            email: value.get("email"),
            name: value.get("name"),
        }
    }
}

pub async fn signup(payload: RegistrationPayload) -> Result<User> {
    let data = serde_json::to_value(payload)?;
    post_request("/auth/register", &data).await
}
