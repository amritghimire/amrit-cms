use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::util::ServiceExt;

use api_server::routes::create_router;

#[tokio::test]
async fn not_found() {
    let app = create_router().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/random_address")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
