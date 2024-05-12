use auth_service::extractors::confirmation::ConfirmationActionType;
use auth_service::helpers::confirmation::get_confirmation;
use auth_service::helpers::user::fetch_user;
use auth_service::router::create_router;
use axum::http::header::AUTHORIZATION;
use axum::http::{HeaderValue, StatusCode};
use axum::response::Response;
use axum::{http, Router};
use chrono::{Duration, Utc};
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;
use utils::state::AppState;
use utils::test;
use uuid::Uuid;

mod common;

#[sqlx::test]
async fn confirm_valid_token(pool: PgPool) {
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");

    let state = AppState::test_state(pool, None);
    let app = create_router().with_state(state);
    let (confirmation, token) =
        common::confirmation_fixture(&mut conn, ConfirmationActionType::UserVerification).await;
    let session_token = common::session_fixture(&mut conn, confirmation.user_id).await;
    let response = send_request(&app, &token, &session_token).await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test]
async fn confirm_unauthorized(pool: PgPool) {
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");

    let state = AppState::test_state(pool, None);
    let app = create_router().with_state(state);
    let (_, token) =
        common::confirmation_fixture(&mut conn, ConfirmationActionType::UserVerification).await;
    let response = send_request(&app, &token, "").await;

    test::assert_response(
        response,
        StatusCode::UNAUTHORIZED,
        "Authorization token invalid: token not available",
    )
    .await;
}

#[sqlx::test]
async fn confirm_invalid_session_token(pool: PgPool) {
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");
    let identifier = Uuid::new_v4();
    let session_token = format!("{}.verifier_hash", identifier);

    let state = AppState::test_state(pool, None);
    let app = create_router().with_state(state);
    let (_, token) =
        common::confirmation_fixture(&mut conn, ConfirmationActionType::UserVerification).await;
    let response = send_request(&app, &token, &session_token).await;

    test::assert_response(
        response,
        StatusCode::UNAUTHORIZED,
        "Authorization token invalid: session not found",
    )
    .await;
}

#[sqlx::test]
async fn confirm_different_user_logged_in(pool: PgPool) {
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");

    let state = AppState::test_state(pool, None);
    let app = create_router().with_state(state);
    let (_, token) =
        common::confirmation_fixture(&mut conn, ConfirmationActionType::UserVerification).await;
    let new_user = common::user_fixture(&mut conn).await;
    let session_token = common::session_fixture(&mut conn, new_user.id).await;
    let response = send_request(&app, &token, &session_token).await;

    test::assert_response(
        response,
        StatusCode::UNAUTHORIZED,
        "insufficient permission: login with the user you want to verify",
    )
    .await;
}

#[sqlx::test]
async fn confirm_token_verify_user(pool: PgPool) {
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");

    let state = AppState::test_state(pool, None);
    let app = create_router().with_state(state);
    let (confirmation, token) =
        common::confirmation_fixture(&mut conn, ConfirmationActionType::UserVerification).await;
    let session_token = common::session_fixture(&mut conn, confirmation.user_id).await;
    send_request(&app, &token, &session_token).await;

    let user = fetch_user(&mut conn, confirmation.user_id).await.unwrap();
    assert!(user.is_confirmed);
    assert!(
        get_confirmation(&mut conn, &confirmation.confirmation_id.to_string())
            .await
            .is_err()
    );
}

#[sqlx::test]
async fn confirm_token_verify_user_failed(pool: PgPool) {
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");

    let state = AppState::test_state(pool, None);
    let app = create_router().with_state(state);
    let (confirmation, token) =
        common::confirmation_fixture(&mut conn, ConfirmationActionType::UserVerification).await;
    let session_token = common::session_fixture(&mut conn, confirmation.user_id).await;
    // missing confirmation details
    sqlx::query!(
        "update confirmations set details = null where confirmation_id = $1",
        confirmation.confirmation_id
    )
    .execute(&mut *conn)
    .await
    .expect("Cannot update details");
    let response = send_request(&app, &token, &session_token).await;
    test::assert_response(
        response,
        StatusCode::BAD_REQUEST,
        "Invalid token provided: missing confirmation details",
    )
    .await;

    // missing confirmation email
    sqlx::query!(
        "update confirmations set details = $1 where confirmation_id = $2",
        json!({}),
        confirmation.confirmation_id
    )
    .execute(&mut *conn)
    .await
    .expect("Cannot update details");
    let response = send_request(&app, &token, &session_token).await;
    test::assert_response(
        response,
        StatusCode::BAD_REQUEST,
        "Invalid token provided: missing confirmation email",
    )
    .await;

    // invalid email set
    sqlx::query!(
        "update confirmations set details = $1 where confirmation_id = $2",
        json!({"email": 1}),
        confirmation.confirmation_id
    )
    .execute(&mut *conn)
    .await
    .expect("Cannot update details");
    let response = send_request(&app, &token, &session_token).await;
    test::assert_response(
        response,
        StatusCode::BAD_REQUEST,
        "Invalid token provided: invalid email set",
    )
    .await;

    // email mismatch
    sqlx::query!(
        "update confirmations set details = $1 where confirmation_id = $2",
        json!({"email": "invalid"}),
        confirmation.confirmation_id
    )
    .execute(&mut *conn)
    .await
    .expect("Cannot update details");
    let response = send_request(&app, &token, &session_token).await;
    test::assert_response(
        response,
        StatusCode::BAD_REQUEST,
        "Invalid token provided: email mismatch",
    )
    .await;

    let user = fetch_user(&mut conn, confirmation.user_id).await.unwrap();
    assert!(!user.is_confirmed);
    assert!(
        get_confirmation(&mut conn, &confirmation.confirmation_id.to_string())
            .await
            .is_ok()
    );
}

#[sqlx::test]
async fn confirm_token_invalid(pool: PgPool) {
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");

    let state = AppState::test_state(pool, None);
    let app = create_router().with_state(state);
    let (confirmation, token) =
        common::confirmation_fixture(&mut conn, ConfirmationActionType::UserVerification).await;
    let session_token = common::session_fixture(&mut conn, confirmation.user_id).await;
    // Incomplete token
    let response = send_request(&app, "invalid", &session_token).await;
    test::assert_response(
        response,
        StatusCode::BAD_REQUEST,
        "Invalid token provided: incomplete token",
    )
    .await;
    // Empty token part
    let response = send_request(&app, "invalid.", &session_token).await;
    test::assert_response(
        response,
        StatusCode::BAD_REQUEST,
        "Invalid token provided: empty token part",
    )
    .await;

    // Invalid verifier
    let response = send_request(
        &app,
        &format!("{}.invalid", confirmation.confirmation_id),
        &session_token,
    )
    .await;
    test::assert_response(
        response,
        StatusCode::BAD_REQUEST,
        "Invalid token provided: invalid hash",
    )
    .await;

    // Invalid token type
    sqlx::query!(
        "update confirmations set action_type = 'invalid' where confirmation_id = $1",
        confirmation.confirmation_id
    )
    .execute(&mut *conn)
    .await
    .expect("Cannot update to invalid action type");
    let response = send_request(&app, &token, &session_token).await;
    test::assert_response(
        response,
        StatusCode::BAD_REQUEST,
        "Invalid token provided: invalid token type",
    )
    .await;

    // Expired token
    sqlx::query!(
        "update confirmations set expires_at = $1 where confirmation_id = $2",
        Utc::now() - Duration::try_days(2).unwrap(),
        confirmation.confirmation_id
    )
    .execute(&mut *conn)
    .await
    .expect("Cannot update confirmation to expired");
    let response = send_request(&app, &token, &session_token).await;
    test::assert_response(
        response,
        StatusCode::BAD_REQUEST,
        "Invalid token provided: expired token",
    )
    .await;
}

async fn send_request(app: &Router, token: &str, session_token: &str) -> Response {
    let data = json!({});
    let mut request =
        test::build_request(&format!("/confirm/{}", token), http::Method::POST, &data);
    let session_header = HeaderValue::from_str(session_token).unwrap();
    request.headers_mut().insert(AUTHORIZATION, session_header);
    app.clone().oneshot(request).await.unwrap()
}
