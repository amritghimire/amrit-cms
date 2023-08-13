use std::sync::mpsc;
use axum::body::Body;
use axum::http;
use axum::http::{Request, StatusCode};
use fake::faker::internet::en::SafeEmail;
use fake::faker::name::en::Name;
use fake::Fake;
use serde_json::{json, Value};
use sqlx::PgPool;
use subscription_service::router::create_router;
use tower::util::ServiceExt;
use utils::state::AppState;
use utils::test;

#[sqlx::test]
async fn subscribe_returns_a_200_for_valid_form_data(pool: PgPool) {
    let (tx, _) = mpsc::sync_channel(5);
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");

    let state = test::test_state_for_email(pool, tx);
    let app = create_router().with_state(state);


    let name: String = Name().fake();
    let email: String = SafeEmail().fake();

    let data = json!({
        "name": name,
        "email": email
    });

    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/")
        .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
        .body(Body::from(serde_json::to_vec(&data).unwrap()))
        .unwrap();

    let response = app
        .clone()
        .oneshot(request)
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&mut conn)
        .await
        .expect("Unable to fetch the table");

    assert_eq!(saved.email, email);
    assert_eq!(saved.name, name);

    // let email = rx.recv().unwrap();
    // assert_eq!(email.sender, settings.email.sender);
    // assert_eq!(email.to, recipient_mail);
    // assert_eq!(email.subject, mail_subject);
    // assert_eq!(email.body, mail_body);

    // Add it again
    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/")
        .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
        .body(Body::from(serde_json::to_vec(&data).unwrap()))
        .unwrap();
    let response = app
        .clone()
        .oneshot(request)
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        body,
        json!({ "level": "error","message": "Email already subscribed", "status": 400} )
    );
}

#[sqlx::test]
async fn subscribe_returns_a_400_for_invalid_form_data(pool: PgPool) {
    let state = AppState::test_state(pool, None);
    let app = create_router().with_state(state);

    let test_cases = vec![
        (json!({"name": "Amrit"}), "missing the email"),
        (json!({"email": "test@example.com"}), "missing the name"),
        (json!({}), "missing both name and email"),
        (
            json!({"name": "", "email": "test@example.com"}),
            "empty name provided",
        ),
        (json!({"name":"Amrit", "email":""}), "empty email provided"),
        (json!({"name":"", "email": ""}), "both fields are empty"),
        (
            json!({"name": "(Amrit)", "email": "test@example.com"}),
            "invalid name",
        ),
    ];

    for (payload, error_message) in test_cases {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                    .unwrap(),
            )
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
