use crate::common::user_fixture;
use auth_service::errors::confirm::ConfirmUserError;
use auth_service::extractors::confirmation::{Confirmation, ConfirmationActionType};
use axum::response::Response;
use axum::{http, Router};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::time::Duration;
use tower::ServiceExt;
use url::Url;
use utils::email::get_link;
use utils::test;

mod common;

#[sqlx::test]
async fn initiate_reset_successfully_adds_confirmation_with_email(pool: PgPool) {
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");

    let (_, task_rx, _, app) = common::setup_app_with_task_thread(pool);
    let user = user_fixture(&mut conn).await;

    let email = user.email;
    let request_data = json!({
        "username_or_email": email,
    });
    let response = send_request(&app, "/initiate-reset", &request_data).await;
    assert_eq!(response.status(), http::StatusCode::OK);

    let task = task_rx.try_recv().expect("Task not thrown out.");
    task.handle.await.expect("Join error, task panicked");

    let confirmation = sqlx::query_as!(
        Confirmation,
        r#"
        SELECT confirmation_id, details, verifier_hash, user_id, created_at, expires_at, action_type from confirmations
        "#,
    ).fetch_one(&mut *conn).await.map_err(ConfirmUserError::ConfirmationDatabaseError).expect("Unable to fetch confirmation");

    assert_eq!(confirmation.user_id, user.id);
    assert_eq!(
        confirmation.action_type,
        ConfirmationActionType::PasswordReset
    );
}

#[sqlx::test]
async fn initiate_reset_successfully_adds_confirmation_with_username(pool: PgPool) {
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");

    let (_, task_rx, _, app) = common::setup_app_with_task_thread(pool);
    let user = user_fixture(&mut conn).await;

    let request_data = json!({
        "username_or_email": user.username,
    });
    let response = send_request(&app, "/initiate-reset", &request_data).await;
    assert_eq!(response.status(), http::StatusCode::OK);

    let task = task_rx.try_recv().expect("Task not thrown out.");
    task.handle.await.expect("Join error, task panicked");

    let confirmation = sqlx::query_as!(
        Confirmation,
        r#"
        SELECT confirmation_id, details, verifier_hash, user_id, created_at, expires_at, action_type from confirmations
        "#,
    ).fetch_one(&mut *conn).await.map_err(ConfirmUserError::ConfirmationDatabaseError).expect("Unable to fetch confirmation");

    assert_eq!(confirmation.user_id, user.id);
    assert_eq!(
        confirmation.action_type,
        ConfirmationActionType::PasswordReset
    );
}

#[sqlx::test]
async fn initiate_reset_non_existent_username(pool: PgPool) {
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");

    let (email_rx, task_rx, _, app) = common::setup_app_with_task_thread(pool);
    let request_data = json!({
        "username_or_email": "username",
    });

    let response = send_request(&app, "/initiate-reset", &request_data).await;
    assert_eq!(response.status(), http::StatusCode::OK);

    let task = task_rx.try_recv().expect("Task not thrown out.");
    task.handle.await.expect("Join error, task panicked");

    let confirmation_count = sqlx::query!(
        r#"
        select * from confirmations
        "#
    )
    .fetch_all(&mut *conn)
    .await
    .expect("Unable to select from confirmations");

    assert_eq!(confirmation_count.len(), 0);

    assert!(email_rx.try_recv().is_err());
}

#[sqlx::test]
async fn initiate_reset_successfully_sends_email(pool: PgPool) {
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");

    let (email_rx, task_rx, settings, app) = common::setup_app_with_task_thread(pool);
    let user = user_fixture(&mut conn).await;

    let email = user.email;
    let request_data = json!({
        "username_or_email": email,
    });
    let response = send_request(&app, "/initiate-reset", &request_data).await;
    assert_eq!(response.status(), http::StatusCode::OK);

    let task = task_rx.try_recv().expect("Task not thrown out.");
    task.handle.await.expect("Join error, task panicked");

    let email_object = email_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("Email not sent during the reset");

    assert_eq!(email_object.sender.to_string(), "test@example.com");
    assert_eq!(email_object.to[0].email, email);
    assert_eq!(email_object.subject, "Please proceed to reset the password");
    let raw_link = get_link(&email_object.plain);
    let confirmation_link = Url::parse(&raw_link).unwrap();
    let application_link = Url::parse(&settings.application.full_url()).unwrap();
    assert_eq!(
        confirmation_link.host_str().unwrap(),
        application_link.host_str().unwrap()
    )
}

async fn send_request(app: &Router, path: &str, data: &Value) -> Response {
    let request = test::build_request(path, http::Method::POST, data);
    app.clone().oneshot(request).await.unwrap()
}
