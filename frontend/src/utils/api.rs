use crate::entities::user::User;
use log::info;

pub fn form_url(path: &str) -> String {
    let base_url = web_sys::window().unwrap().location().origin().unwrap();
    format!("{}/api{}", base_url, path)
}

pub async fn me() -> Result<User, reqwest::Error> {
    info!("Starting api call");
    let url = form_url("/auth/me");
    reqwest::get(&url).await?.json().await
}
