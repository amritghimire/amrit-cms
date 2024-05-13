use crate::entities::user::User;
use crate::utils::api::post_request;
use crate::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InitiateResetResponse {}

#[derive(Debug, Deserialize, Serialize, Clone, PartialOrd, PartialEq)]
pub struct CheckResetTokenResponse {
    pub name: String,
    pub username: String,
}

pub async fn initiate_reset(username_or_email: String) -> Result<InitiateResetResponse> {
    let data = json!({
        "username_or_email": username_or_email,
    });
    post_request("/auth/initiate-reset", &data).await
}

pub async fn check_reset_token(token: String) -> Result<CheckResetTokenResponse> {
    let data = json!({});

    post_request(&format!("/auth/check-reset/{}", token), &data).await
}

pub async fn reset_password(token: &str, password: String, new_password: String) -> Result<User> {
    let data = json!({
        "password": password,
        "confirm_password": new_password
    });
    post_request(&format!("/auth/reset-password/{}", token), &data).await
}
