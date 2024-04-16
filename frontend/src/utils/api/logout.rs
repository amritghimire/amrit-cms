use crate::utils::api::post_request;
use crate::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LogoutResponse {
    ok: bool,
}

pub async fn logout() -> Result<LogoutResponse> {
    let data = json!({});
    post_request("/auth/logout", &data).await
}
