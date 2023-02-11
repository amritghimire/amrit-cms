use axum::body::Body;
use axum::http;
use axum::http::{Request, StatusCode};
use serde_json::json;
use sqlx::{Connection, PgConnection};
use subscription_service::router::create_router;
use tower::util::ServiceExt;
use utils::configuration::Settings;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = create_router();
    let connection_string = Settings::get_config("test").expect("Test config not available").database.connection_string();
    assert_eq!(connection_string, "");
    let connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres.");

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
    //
    // let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    // let body: Value = serde_json::from_slice(&body).unwrap();
    // assert_eq!(body, json!({ "data": [1, 2, 3, 4] }));
}

#[tokio::test]
async fn subscribe_returns_a_400_for_invalid_form_data() {
    let app = create_router();

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
