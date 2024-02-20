use auth_service::extractor::{Confirmation, ConfirmationActionType, User};
use auth_service::helpers::confirmation::{add_confirmation, get_confirmation};
use auth_service::helpers::user::{fetch_user, insert_user};
use auth_service::payload::RegisterPayload;
use auth_service::router::create_router;
use axum::http::StatusCode;
use axum::response::Response;
use axum::{http, Router};
use chrono::{Duration, Utc};
use fake::faker::internet::en::{SafeEmail, Username};
use fake::faker::name::en::Name;
use fake::Fake;
use serde_json::json;
use sqlx::{PgConnection, PgPool};
use tower::ServiceExt;
use utils::state::AppState;
use utils::test;

#[sqlx::test]
async fn confirm_valid_token(pool: PgPool) {
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");

    let state = AppState::test_state(pool, None);
    let app = create_router().with_state(state);
    let (_, token) = confirmation_fixture(&mut conn).await;
    let response = send_request(&app, &token).await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test]
async fn confirm_token_verify_user(pool: PgPool) {
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");

    let state = AppState::test_state(pool, None);
    let app = create_router().with_state(state);
    let (confirmation, token) = confirmation_fixture(&mut conn).await;
    send_request(&app, &token).await;

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
    let (confirmation, token) = confirmation_fixture(&mut conn).await;

    // missing confirmation details
    sqlx::query!(
        "update confirmations set details = null where confirmation_id = $1",
        confirmation.confirmation_id
    )
    .execute(&mut *conn)
    .await
    .expect("Cannot update details");
    let response = send_request(&app, &token).await;
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
    let response = send_request(&app, &token).await;
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
    let response = send_request(&app, &token).await;
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
    let response = send_request(&app, &token).await;
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
    let (confirmation, token) = confirmation_fixture(&mut conn).await;

    // Incomplete token
    let response = send_request(&app, "invalid").await;
    test::assert_response(
        response,
        StatusCode::BAD_REQUEST,
        "Invalid token provided: incomplete token",
    )
    .await;
    // Empty token part
    let response = send_request(&app, "invalid.").await;
    test::assert_response(
        response,
        StatusCode::BAD_REQUEST,
        "Invalid token provided: empty token part",
    )
    .await;

    // Invalid verifier
    let response = send_request(&app, &format!("{}.invalid", confirmation.confirmation_id)).await;
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
    let response = send_request(&app, &token).await;
    test::assert_response(
        response,
        StatusCode::BAD_REQUEST,
        "Invalid token provided: invalid token type",
    )
    .await;

    // Expired token
    sqlx::query!(
        "update confirmations set expires_at = $1 where confirmation_id = $2",
        Utc::now() - Duration::days(2),
        confirmation.confirmation_id
    )
    .execute(&mut *conn)
    .await
    .expect("Cannot update confirmation to expired");
    let response = send_request(&app, &token).await;
    test::assert_response(
        response,
        StatusCode::BAD_REQUEST,
        "Invalid token provided: expired token",
    )
    .await;
}

async fn send_request(app: &Router, token: &str) -> Response {
    let data = json!({});
    let request = test::build_request(&format!("/confirm/{}", token), http::Method::GET, &data);
    app.clone().oneshot(request).await.unwrap()
}

async fn confirmation_fixture(transaction: &mut PgConnection) -> (Confirmation, String) {
    let user_payload = RegisterPayload {
        username: Username().fake(),
        password: "r0sebudmaelstrom11/20/91aaaa".to_string(),
        email: SafeEmail().fake(),
        confirm_password: "r0sebudmaelstrom11/20/91aaaa".to_string(),
        name: Name().fake(),
    };

    let user = User::try_from(user_payload).expect("Cannot form new user");
    let id = insert_user(transaction, &user)
        .await
        .expect("Cannot insert user");
    let (confirmation, token) = Confirmation::new(
        id,
        ConfirmationActionType::UserVerification,
        json!({"email": user.email}),
    );
    add_confirmation(transaction, &confirmation)
        .await
        .expect("Cannot add confirmation");
    (confirmation, token)
}
