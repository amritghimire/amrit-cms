use axum::{http, Router};
use email_clients::email::EmailObject;
use fake::faker::internet::en::SafeEmail;
use fake::faker::name::en::Name;
use fake::Fake;
use serde_json::json;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use tower::ServiceExt;
use url::Url;
use utils::email::get_link;
use utils::state::BackgroundTask;
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

pub async fn get_confirmation_link(
    rx: &Receiver<EmailObject>,
    app: &Router,
    task_rx: Option<&Receiver<BackgroundTask>>,
) -> String {
    let name: String = Name().fake();
    let email: String = SafeEmail().fake();
    let data = json!({
        "name": name,
        "email": email
    });
    let request = test::build_request("/", http::Method::POST, &data);

    app.clone().oneshot(request).await.unwrap();
    if let Some(task_rx) = task_rx {
        let task = task_rx.try_recv().expect("Task not thrown out.");
        task.handle.await.expect("Join error, task panicked");
    }

    let email_object = rx
        .try_recv()
        .expect("Email not sent during the subscription");
    get_link(&email_object.plain)
}

#[allow(dead_code)]
pub async fn create_confirmed_subscriber(
    rx: &Receiver<EmailObject>,
    app: &Router,
    task_rx: Option<&Receiver<BackgroundTask>>,
) {
    let raw_link = get_confirmation_link(rx, app, task_rx).await;
    let token = extract_token(raw_link);
    let url = format!("/confirm?token={}", token);
    let data = json!({});

    let request = test::build_request(url.as_str(), http::Method::GET, &data);

    app.clone().oneshot(request).await.unwrap();
}
