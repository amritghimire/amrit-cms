#[path = "./helper.rs"]
mod helper;

use crate::helper::{create_confirmed_subscriber, get_confirmation_link};
use axum::http;
use axum::http::StatusCode;
use claims::assert_err;
use serde_json::json;
use sqlx::PgPool;
use std::sync::mpsc;
use subscription_service::router::create_router;
use tower::ServiceExt;
use utils::test;

#[sqlx::test]
async fn newsletter_are_not_delivered_to_unconfirmed_subscriber(pool: PgPool) {
    let (tx, rx) = mpsc::sync_channel(5);
    let state = test::test_state_for_email(pool, tx);
    let app = create_router().with_state(state);

    get_confirmation_link(&rx, &app).await; // Create unconfirmed user
    create_confirmed_subscriber(&rx, &app).await;

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

    assert_eq!(rx.try_iter().count(), 1); // Assert only one email is sent.
}
