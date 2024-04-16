use crate::utils::api::post_request;
use crate::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConfirmResponse {}

pub async fn confirm_token(token: &str) -> Result<ConfirmResponse> {
    let data = json!({});
    post_request(&format!("/auth/confirm/{}", token), &data).await
}
