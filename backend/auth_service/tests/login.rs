use auth_service::extractors::session::SESSION_TOKEN_COOKIE;
use auth_service::router::create_router;
use axum::http::header::{AUTHORIZATION, SET_COOKIE};
use axum::http::StatusCode;
use axum::response::Response;
use axum::{http, Router};
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;
use utils::state::AppState;
use utils::test;

mod common;

#[sqlx::test]
async fn login_successful(pool: PgPool) {
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");

    let state = AppState::test_state(pool, None);
    let app = create_router().with_state(state);

    let user = common::user_fixture(&mut conn).await;
    let response = send_request(&app, &user.username, common::STRONG_PASSWORD).await;

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
    assert!(cookie_header.contains(SESSION_TOKEN_COOKIE))
}

#[sqlx::test]
async fn login_username_not_found(pool: PgPool) {
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");

    let state = AppState::test_state(pool, None);
    let app = create_router().with_state(state);

    let _ = common::user_fixture(&mut conn).await;
    let response = send_request(&app, "invalid!", common::STRONG_PASSWORD).await;
    test::assert_response(
        response,
        StatusCode::BAD_REQUEST,
        "login failed: username not found",
    )
    .await;
}

#[sqlx::test]
async fn login_username_invalid_password(pool: PgPool) {
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");

    let state = AppState::test_state(pool, None);
    let app = create_router().with_state(state);

    let user = common::user_fixture(&mut conn).await;
    let response = send_request(&app, &user.username, "invalid!").await;
    test::assert_response(
        response,
        StatusCode::BAD_REQUEST,
        "login failed: username or password is incorrect",
    )
    .await;
}

#[sqlx::test]
async fn login_returns_a_400_for_invalid_form_data(pool: PgPool) {
    let (_, _, app) = common::setup_app(pool);

    let test_cases = vec![
        (json!({}), "empty payload"),
        (
            json!({"password": common::STRONG_PASSWORD}),
            "missing the username",
        ),
        (json!({"username": "username"}), "missing the password"),
        (
            json!({"username": "sn", "password": common::STRONG_PASSWORD}),
            "short username",
        ),
        (json!({"username": "valid", "password": "small"}), "short "),
    ];

    for (payload, error_message) in test_cases {
        let response = app
            .clone()
            .oneshot(test::build_request("/login", http::Method::POST, &payload))
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

async fn send_request(app: &Router, username: &str, password: &str) -> Response {
    let data = json!({
        "username": username,
        "password": password
    });
    let request = test::build_request("/login", http::Method::POST, &data);
    app.clone().oneshot(request).await.unwrap()
}
