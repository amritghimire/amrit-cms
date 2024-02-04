#[path = "./helper.rs"]
mod helper;

use crate::helper::{extract_token, get_confirmation_link};
use axum::response::Response;
use axum::{http, Router};
use serde_json::json;
use sqlx::PgPool;
use std::sync::mpsc;
use subscription_service::router::create_router;
use tower::util::ServiceExt;
use url::{Position, Url};
use utils::configuration::{RunMode, Settings};
use utils::test;

#[sqlx::test]
async fn confirmations_without_token_are_rejected_with_400(pool: PgPool) {
    let (tx, _rx) = mpsc::sync_channel(5);
    let state = test::test_state_for_email(pool, tx);
    let app = create_router().with_state(state);

    let response = send_request(&app, None).await;

    assert_eq!(response.status(), http::StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn link_returned_by_subscribe_returns_200_if_called(pool: PgPool) {
    let (tx, rx) = mpsc::sync_channel(5);
    let settings = Settings::get_config(RunMode::Test).expect("Unable to fetch test config");
    let state = test::test_state_for_email(pool, tx);
    let app = create_router().with_state(state);

    let raw_link = get_confirmation_link(&rx, &app).await;
    let confirmation_link = Url::parse(&raw_link).unwrap();
    let application_link = Url::parse(&settings.application.full_url()).unwrap();
    assert_eq!(
        confirmation_link.host_str().unwrap(),
        application_link.host_str().unwrap()
    );

    let url_part = confirmation_link[Position::BeforePath..]
        .strip_prefix("/subscription")
        .unwrap();

    let data = json!({});
    let request = test::build_request(url_part, http::Method::GET, &data);

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), http::StatusCode::OK);
}

#[sqlx::test]
async fn clicking_on_confirmation_link_confirms_a_subscriber(pool: PgPool) {
    let (tx, rx) = mpsc::sync_channel(5);
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");
    let state = test::test_state_for_email(pool, tx);
    let app = create_router().with_state(state);

    let raw_link = get_confirmation_link(&rx, &app).await;
    let token = extract_token(raw_link);
    let response = send_request(&app, Some(&token)).await;

    assert_eq!(response.status(), http::StatusCode::OK);
    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&mut *conn)
        .await
        .expect("Unable to fetch the table");

    assert_eq!(saved.status, "confirmed");
}

async fn send_request(app: &Router, token: Option<&str>) -> Response {
    let mut url = "/confirm".to_string();
    if let Some(token) = token {
        url = format!("{}?token={}", url, token)
    }
    let data = json!({});
    let request = test::build_request(url.as_str(), http::Method::GET, &data);

    app.clone().oneshot(request).await.unwrap()
}
