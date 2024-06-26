use axum::http::StatusCode;
use axum::response::Response;
use axum::{http, Router};
use fake::faker::internet::en::SafeEmail;
use fake::faker::name::en::Name;
use fake::Fake;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::mpsc;

use http_body_util::BodyExt;
use subscription_service::router::create_router;
use tower::util::ServiceExt;
use url::Url;
use utils::configuration::{RunMode, Settings};
use utils::email::get_link;
use utils::state::AppState;
use utils::test;

#[sqlx::test]
async fn subscribe_200_for_valid_form_data(pool: PgPool) {
    let (tx, _rx) = mpsc::sync_channel(5);
    let state = test::test_state_for_email(pool, tx);
    let app = create_router().with_state(state);

    let name: String = Name().fake();
    let email: String = SafeEmail().fake();

    let response = send_request(&app, &name, &email).await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test]
async fn subscribe_valid_form_data_is_inserted(pool: PgPool) {
    let (tx, _rx) = mpsc::sync_channel(5);
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");
    let state = test::test_state_for_email(pool, tx);
    let app = create_router().with_state(state);

    let name: String = Name().fake();
    let email: String = SafeEmail().fake();

    send_request(&app, &name, &email).await;

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&mut *conn)
        .await
        .expect("Unable to fetch the table");

    assert_eq!(saved.email, email);
    assert_eq!(saved.name, name);
    assert_eq!(saved.status, "pending");
}

#[sqlx::test]
async fn subscribe_valid_form_email_sent(pool: PgPool) {
    let (email_tx, email_rx) = mpsc::sync_channel(5);
    let (task_tx, task_rx) = mpsc::sync_channel(5);

    let settings = Settings::get_config(RunMode::Test).expect("Unable to fetch test config");
    let mut state = test::test_state_for_email(pool, email_tx);
    state.tasks = Some(task_tx);

    let app = create_router().with_state(state);

    let name: String = Name().fake();
    let email: String = SafeEmail().fake();

    send_request(&app, &name, &email).await;
    let task = task_rx.try_recv().expect("Task not thrown out.");
    task.handle.await.expect("Join error, task panicked");

    let email_object = email_rx
        .try_recv()
        .expect("Email not sent during the subscription");
    assert_eq!(email_object.sender.to_string(), "test@example.com");
    assert_eq!(email_object.to[0].email, email);
    assert_eq!(email_object.subject, "Welcome to our newsletter!");
    let raw_link = get_link(&email_object.plain);
    let confirmation_link = Url::parse(&raw_link).unwrap();
    let application_link = Url::parse(&settings.application.full_url()).unwrap();
    assert_eq!(
        confirmation_link.host_str().unwrap(),
        application_link.host_str().unwrap()
    )
}

#[sqlx::test]
async fn subscribe_valid_form_already_subscribed(pool: PgPool) {
    let (tx, _rx) = mpsc::sync_channel(5);
    let state = test::test_state_for_email(pool, tx);
    let app = create_router().with_state(state);

    let name: String = Name().fake();
    let email: String = SafeEmail().fake();

    send_request(&app, &name, &email).await;

    // Send it again
    let response = send_request(&app, &name, &email).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        body,
        json!({"details": {}, "level": "error","message": "Email already subscribed", "status": 400} )
    );
}

#[sqlx::test]
async fn subscribe_returns_a_400_for_invalid_form_data(pool: PgPool) {
    let state = AppState::test_state(pool, None);
    let app = create_router().with_state(state);

    let test_cases = vec![
        (json!({"name": "Amrit"}), "missing the email"),
        (json!({"email": "test@example.com"}), "missing the name"),
        (json!({}), "missing both name and email"),
        (
            json!({"name": "", "email": "test@example.com"}),
            "empty name provided",
        ),
        (json!({"name":"Amrit", "email":""}), "empty email provided"),
        (json!({"name":"", "email": ""}), "both fields are empty"),
        (
            json!({"name": "(Amrit)", "email": "test@example.com"}),
            "invalid name",
        ),
    ];

    for (payload, error_message) in test_cases {
        let response = app
            .clone()
            .oneshot(test::build_request("/", http::Method::POST, &payload))
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "The request didn't throw 400 request for the case when {}",
            error_message
        );
    }
}

async fn send_request(app: &Router, name: &str, email: &str) -> Response {
    let data = json!({
        "name": name,
        "email": email
    });

    let request = test::build_request("/", http::Method::POST, &data);

    app.clone().oneshot(request).await.unwrap()
}
