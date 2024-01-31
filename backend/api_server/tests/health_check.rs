use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use std::net::SocketAddr;
use tower::util::ServiceExt; // for `collect`

use api_server::routes::create_router;

#[tokio::test]
async fn health_check() {
    let app = create_router().await;

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

    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"Ok");
}

// You can also spawn a server and talk to it like any other HTTP server:
#[tokio::test]
async fn check_for_server() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap())
        .await
        .unwrap();

    let addr = listener.local_addr().unwrap();
    let app = create_router().await;

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .build_http();

    let response = client
        .request(
            Request::builder()
                .uri(format!("http://{}/health_check", addr))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"Ok");
}
