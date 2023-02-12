use axum::body::Body;
use axum::http;
use axum::http::{Request, StatusCode};
use serde_json::json;
use sqlx::PgPool;
use subscription_service::router::create_router;
use tower::util::ServiceExt;
use utils::state::AppState;

#[sqlx::test]
async fn subscribe_returns_a_200_for_valid_form_data(pool: PgPool) {
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");
    let state = AppState::test_state(pool);
    let app = create_router().with_state(state);

    let data = json!({
        "name": "Amrit",
        "email": "test@example.com"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(serde_json::to_vec(&data).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&mut conn)
        .await
        .expect("Unable to fetch the table");

    assert_eq!(saved.email, "test@example.com");
    assert_eq!(saved.name, "Amrit");
    //
    // let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    // let body: Value = serde_json::from_slice(&body).unwrap();
    // assert_eq!(body, json!({ "data": [1, 2, 3, 4] }));
}

#[sqlx::test]
async fn subscribe_returns_a_400_for_invalid_form_data(pool: PgPool) {
    let state = AppState::test_state(pool);
    let app = create_router().with_state(state);

    let test_cases = vec![
        (json!({"name": "Amrit"}), "missing the email"),
        (json!({"email": "test@example.com"}), "missing the name"),
        (json!({}), "missing both name and email"),
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
