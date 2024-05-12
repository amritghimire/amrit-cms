use crate::common::user_fixture;
use auth_service::extractors::confirmation::{Confirmation, ConfirmationActionType};
use axum::http::header::AUTHORIZATION;
use axum::http::{HeaderValue, StatusCode};
use axum::response::Response;
use axum::{http, Router};
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;
use url::Url;
use utils::email::get_link;
use utils::test;

mod common;

#[sqlx::test]
async fn resend_verification_email(pool: PgPool) {
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");

    let (rx, settings, app) = common::setup_app(pool);

    let user = user_fixture(&mut conn).await;
    let session_token = common::session_fixture(&mut conn, user.id).await;

    let response = send_request(&app, &session_token).await;

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "The request didn't get 200 response for resend verification"
    );

    let result = sqlx::query_as!(
        Confirmation,
        r#"SELECT confirmation_id, details, verifier_hash, user_id, created_at, expires_at, action_type from confirmations"#
    ).fetch_one(&mut *conn)
        .await
        .expect("Unable to fetch confirmations objects");

    assert_eq!(result.action_type, ConfirmationActionType::UserVerification);

    let email_object = rx
        .try_recv()
        .expect("Email not sent during the subscription");
    assert_eq!(email_object.sender.to_string(), "test@example.com");
    assert_eq!(email_object.to[0].email, user.email);
    assert_eq!(
        email_object.subject,
        "Please verify your account to proceed"
    );
    let raw_link = get_link(&email_object.plain);
    let confirmation_link = Url::parse(&raw_link).unwrap();
    let application_link = Url::parse(&settings.application.full_url()).unwrap();
    assert_eq!(
        confirmation_link.host_str().unwrap(),
        application_link.host_str().unwrap()
    )
}

async fn send_request(app: &Router, session_token: &str) -> Response {
    let data = json!({});
    let mut request = test::build_request("/resend-verification", http::Method::POST, &data);
    let session_header = HeaderValue::from_str(session_token).unwrap();
    request.headers_mut().insert(AUTHORIZATION, session_header);
    app.clone().oneshot(request).await.unwrap()
}
