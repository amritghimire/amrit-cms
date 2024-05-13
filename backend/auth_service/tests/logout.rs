use auth_service::router::create_router;
use axum::http::header::AUTHORIZATION;
use axum::http::{HeaderValue, StatusCode};
use axum::response::Response;
use axum::{http, response, Router};
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;
use utils::state::AppState;
use utils::test;
use uuid::Uuid;

mod common;

#[sqlx::test]
async fn logout(pool: PgPool) {
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");
    let state = AppState::test_state(pool, None);
    let app = create_router().with_state(state);

    let user = common::user_fixture(&mut conn).await;
    let session_token = common::session_fixture(&mut conn, user.id).await;

    let response = send_request(&app, &session_token).await;
    assert_eq!(response.status(), StatusCode::OK);

    let sessions = sqlx::query!(
        r#"SELECT COUNT(*) as count FROM sessions WHERE user_id = $1"#,
        user.id,
    )
    .fetch_one(&mut *conn)
    .await
    .expect("Unable to fetch sessions");

    assert_eq!(sessions.count, Some(0));
}

async fn send_request(app: &Router, session_token: &str) -> Response {
    let data = json!({});
    let mut request = test::build_request("/logout", http::Method::POST, &data);
    let session_header = HeaderValue::from_str(session_token).unwrap();
    request.headers_mut().insert(AUTHORIZATION, session_header);
    app.clone().oneshot(request).await.unwrap()
}
