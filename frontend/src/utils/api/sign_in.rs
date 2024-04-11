use crate::entities::user::User;
use crate::utils::api::post_request;
use crate::Result;
use serde_json::json;

pub async fn signin(username: String, password: String) -> Result<User> {
    let data = json!({
        "username": username,
        "password": password
    });
    post_request("/auth/login", &data).await
}
