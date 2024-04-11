use crate::entities::user::User;
use log::info;

use crate::utils::api;
use crate::Result;

pub async fn me() -> Result<User> {
    info!("Starting api call");
    let url = api::form_url("/auth/me");
    let response = reqwest::get(&url).await?;
    let user = api::process_response(response).await?;
    Ok(user)
}
