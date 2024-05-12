use axum::http::StatusCode;
use axum::response::Response;
use axum::{http, Router};

use auth_service::extractors::confirmation::ConfirmationActionType;
use auth_service::router::create_router;
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;
use utils::state::AppState;
use utils::test;

mod common;

#[sqlx::test]
async fn reset_check_valid_token(pool: PgPool) {
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");

    let state = AppState::test_state(pool, None);
    let app = create_router().with_state(state);
    let (_, token) =
        common::confirmation_fixture(&mut conn, ConfirmationActionType::PasswordReset).await;

    let response = send_request(&app, &token).await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test]
async fn reset_check_invalid_token_type(pool: PgPool) {
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");

    let state = AppState::test_state(pool, None);
    let app = create_router().with_state(state);
    let (_, _) =
        common::confirmation_fixture(&mut conn, ConfirmationActionType::PasswordReset).await;
    let (_, token) =
        common::confirmation_fixture(&mut conn, ConfirmationActionType::UserVerification).await;

    let response = send_request(&app, &token).await;

    test::assert_response(response, StatusCode::BAD_REQUEST, "Invalid action type").await;
}

async fn send_request(app: &Router, token: &str) -> Response {
    let data = json!({});
    let request = test::build_request(
        &format!("/check-reset/{}", token),
        http::Method::POST,
        &data,
    );
    app.clone().oneshot(request).await.unwrap()
}
