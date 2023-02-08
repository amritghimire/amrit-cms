use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::util::ServiceExt;

use api_server::routes::create_router;

#[tokio::test]
async fn hello_world() {
    let app = create_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health_check")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"Ok");
}
