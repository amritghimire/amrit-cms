use crate::configuration::{RunMode, Settings};
use crate::email::{EmailClient, EmailObject, MessagePassingClient};
use crate::state::AppState;
use axum::body::Body;
use axum::http;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::mpsc::SyncSender;

pub fn test_state_for_email(pool: PgPool, tx: SyncSender<EmailObject>) -> AppState {
    let settings = Settings::get_config(RunMode::Test).expect("Unable to fetch test config");

    let email_client = EmailClient::MessagePassingClient(MessagePassingClient::with_tx(
        settings.email.clone(),
        tx,
    ));
    AppState::test_email_state(pool, email_client)
}

pub fn build_request(url: &str, method: http::Method, data: &Value) -> Request<Body> {
    let request = Request::builder()
        .method(method)
        .uri(url)
        .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
        .body(Body::from(serde_json::to_vec(&data).unwrap()))
        .unwrap();
    request
}

pub async fn assert_response(response: Response, status_code: StatusCode, message: &str) {
    assert_eq!(response.status(), status_code);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body["message"], json!(message));
    assert_eq!(body["status"], json!(status_code.as_u16()));
}
