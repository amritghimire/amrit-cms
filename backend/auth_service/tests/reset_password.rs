use auth_service::extractors::confirmation::ConfirmationActionType;
use auth_service::extractors::session::SESSION_TOKEN_COOKIE;
use auth_service::helpers::user::fetch_user;
use axum::http::header::{AUTHORIZATION, SET_COOKIE};
use axum::http::StatusCode;
use axum::response::Response;
use axum::{http, Router};
use secrecy::ExposeSecret;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::time::Duration;
use tower::ServiceExt;
use utils::test;
use uuid::Uuid;

mod common;

pub static NEW_PASSWORD: &str = "r0sebudmaelstrom11/20/91bbbb";

#[sqlx::test]
async fn reset_password_valid_token_200_response(pool: PgPool) {
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");

    let (_, _, app) = common::setup_app(pool);

    let (_, token) =
        common::confirmation_fixture(&mut conn, ConfirmationActionType::PasswordReset).await;

    let request_data = json!({
        "password": NEW_PASSWORD,
        "confirm_password": NEW_PASSWORD
    });

    // Check that the reset is successful with 200 response.
    let response = send_request(&app, &token, request_data).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test]
async fn reset_password_valid_token_clears_resets(pool: PgPool) {
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");

    let (_, _, app) = common::setup_app(pool);

    let (confirmation, token) =
        common::confirmation_fixture(&mut conn, ConfirmationActionType::PasswordReset).await;

    let request_data = json!({
        "password": NEW_PASSWORD,
        "confirm_password": NEW_PASSWORD
    });

    send_request(&app, &token, request_data).await;

    let resets = sqlx::query!(
        r#"SELECT COUNT(*) as count FROM confirmations WHERE user_id = $1 AND action_type = $2"#,
        confirmation.user_id,
        String::from(ConfirmationActionType::PasswordReset)
    )
    .fetch_one(&mut *conn)
    .await
    .expect("Unable to fetch sessions");

    assert_eq!(resets.count, Some(0));
}

#[sqlx::test]
async fn reset_password_valid_token_old_session_cleared(pool: PgPool) {
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");

    let (_, _, app) = common::setup_app(pool);

    let (confirmation, token) =
        common::confirmation_fixture(&mut conn, ConfirmationActionType::PasswordReset).await;
    let old_session = common::session_fixture(&mut conn, confirmation.user_id).await;

    let request_data = json!({
        "password": NEW_PASSWORD,
        "confirm_password": NEW_PASSWORD
    });

    send_request(&app, &token, request_data).await;

    // Verify old sessions are cleared
    let (identifier, _) = old_session.split_once('.').expect("Incomplete token");
    let identifier = Uuid::parse_str(identifier).expect("Invalid UUID");
    let sessions = sqlx::query!(
        r#"SELECT COUNT(*) as count FROM sessions WHERE user_id = $1 AND identifier = $2"#,
        confirmation.user_id,
        identifier
    )
    .fetch_one(&mut *conn)
    .await
    .expect("Unable to fetch sessions");

    assert_eq!(sessions.count, Some(0));
}

#[sqlx::test]
async fn reset_password_valid_token_new_session_response(pool: PgPool) {
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");

    let (_, _, app) = common::setup_app(pool);

    let (confirmation, token) =
        common::confirmation_fixture(&mut conn, ConfirmationActionType::PasswordReset).await;

    let request_data = json!({
        "password": NEW_PASSWORD,
        "confirm_password": NEW_PASSWORD
    });

    let response = send_request(&app, &token, request_data).await;

    // Verify new session is set to the header and database.
    let result = sqlx::query!(
        r#"SELECT identifier FROM sessions WHERE user_id = $1"#,
        confirmation.user_id,
    )
    .fetch_one(&mut *conn)
    .await
    .expect("Unable to fetch session identifier");

    let new_session_identifier: Uuid = result.identifier;

    // Verify session token is set
    let authorization_header: &str = response
        .headers()
        .get(AUTHORIZATION)
        .unwrap()
        .to_str()
        .unwrap();
    let cookie_header: &str = response
        .headers()
        .get(SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(!authorization_header.is_empty());
    assert!(cookie_header.contains(SESSION_TOKEN_COOKIE));

    let (identifier, _) = authorization_header
        .split_once('.')
        .expect("Invalid authorization header");
    assert_eq!(identifier, new_session_identifier.to_string());
}

#[sqlx::test]
async fn reset_password_valid_token_password_hash(pool: PgPool) {
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");
    let (_, _, app) = common::setup_app(pool);

    let (confirmation, token) =
        common::confirmation_fixture(&mut conn, ConfirmationActionType::PasswordReset).await;

    let user = fetch_user(&mut conn, confirmation.user_id)
        .await
        .expect("Unable to fetch user");
    let previous_password_hash = user.password_hash.clone();

    let request_data = json!({
        "password": NEW_PASSWORD,
        "confirm_password": NEW_PASSWORD
    });

    // Check that the reset is successful with 200 response.
    send_request(&app, &token, request_data).await;

    // Verify password is changed.
    let user = fetch_user(&mut conn, confirmation.user_id)
        .await
        .expect("Unable to fetch user");
    let new_password_hash = user.password_hash.clone();
    assert_ne!(
        previous_password_hash.expose_secret(),
        new_password_hash.expose_secret()
    );
    assert!(user.check_password(NEW_PASSWORD));
}

#[sqlx::test]
async fn reset_password_valid_token_email_sent(pool: PgPool) {
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");
    let (email_rx, task_rx, _, app) = common::setup_app_with_task_thread(pool);
    let (confirmation, token) =
        common::confirmation_fixture(&mut conn, ConfirmationActionType::PasswordReset).await;

    let request_data = json!({
        "password": NEW_PASSWORD,
        "confirm_password": NEW_PASSWORD
    });

    send_request(&app, &token, request_data).await;

    let user = fetch_user(&mut conn, confirmation.user_id)
        .await
        .expect("Unable to fetch user");

    // Check if the email is sent or not
    let task = task_rx.try_recv().expect("Task not thrown out.");
    task.handle.await.expect("Join error, task panicked");
    let email_object = email_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("Email not sent during the reset");

    assert_eq!(email_object.to[0].email, user.email);
    assert_eq!(email_object.subject, "Your password was reset recently");
    assert!(email_object
        .plain
        .contains("Your password was successfully reset recently"));
    assert!(email_object
        .html
        .contains("Your password was successfully reset recently"));
}

async fn send_request(app: &Router, token: &str, data: Value) -> Response {
    let request = test::build_request(
        &format!("/reset-password/{}", token),
        http::Method::POST,
        &data,
    );
    app.clone().oneshot(request).await.unwrap()
}
