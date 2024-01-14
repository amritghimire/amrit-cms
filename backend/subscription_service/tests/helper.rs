use axum::{http, Router};
use fake::faker::internet::en::SafeEmail;
use fake::faker::name::en::Name;
use fake::Fake;
use serde_json::json;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use subscription_service::helper::get_link;
use tower::ServiceExt;
use url::Url;
use utils::email::EmailObject;
use utils::test;

pub fn extract_token(raw_link: String) -> String {
    let hash_query: HashMap<_, _> = Url::parse(&raw_link)
        .unwrap()
        .query_pairs()
        .into_owned()
        .collect();
    let token = hash_query.get("token").unwrap();
    token.to_string()
}

pub async fn get_confirmation_link(rx: &Receiver<EmailObject>, app: &Router) -> String {
    let name: String = Name().fake();
    let email: String = SafeEmail().fake();
    let data = json!({
        "name": name,
        "email": email
    });
    let request = test::build_request("/", http::Method::POST, &data);

    app.clone().oneshot(request).await.unwrap();
    let email_object = rx
        .try_recv()
        .expect("Email not sent during the subscription");
    get_link(&email_object.body)
}

pub async fn create_confirmed_subscriber(rx: &Receiver<EmailObject>, app: &Router) {
    let raw_link = get_confirmation_link(rx, &app).await;
    let token = extract_token(raw_link);
    let url = format!("/confirm?token={}", token);
    let data = json!({});

    let request = test::build_request(url.as_str(), http::Method::GET, &data);

    app.clone().oneshot(request).await.unwrap();
}
