use axum::Router;
use std::net::SocketAddr;
use tokio::net::TcpListener;

mod handlers;
pub mod routes;
pub mod telemetry;
mod apps;

pub async fn run(app: Router, addr: SocketAddr) {
    tracing::info!("Starting server in http://{}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
