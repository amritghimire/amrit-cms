use axum::http::StatusCode;
use axum::response::Response;
use axum::{http, Router};
use claims::{assert_err, assert_ok};
use fake::faker::internet::en::SafeEmail;
use fake::faker::name::en::Name;
use fake::Fake;
use serde_json::json;
use sqlx::PgPool;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use subscription_service::router::create_router;
use tower::ServiceExt;
use utils::configuration::{RunMode, Settings};
use utils::email::EmailObject;
use utils::test;

#[sqlx::test]
async fn newsletter_are_not_delivered_to_unconfirmed_subscriber(pool: PgPool) {
    let (tx, rx) = mpsc::sync_channel(5);
    let state = test::test_state_for_email(pool, tx);
    let app = create_router().with_state(state);

    create_unconfirmed_subscriber(&app, &rx).await;
    // Assert no email is sent so far
    assert_err!(rx.try_recv());
    let newsletter_request_body = json!({
        "title": "Newsletter title",
        "content": "Newsletter body as plain text"
    });

    let response = app
        .clone()
        .oneshot(test::build_request(
            "/newsletter",
            http::Method::POST,
            &newsletter_request_body,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

async fn create_unconfirmed_subscriber(app: &Router, rx: &Receiver<EmailObject>) {
    let name: String = Name().fake();
    let email: String = SafeEmail().fake();

    let data = json!({
        "name": name,
        "email": email
    });

    let request = test::build_request("/", http::Method::POST, &data);

    app.clone().oneshot(request).await.unwrap();
    // Discard a email for confirmation from email object.
    assert_ok!(rx.try_recv());
}
