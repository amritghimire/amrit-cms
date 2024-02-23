use axum::Router;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[cfg(feature = "shuttle")]
use sqlx::PgPool;

mod apps;
mod handlers;
pub mod macros;
pub mod migrate;
pub mod routes;
pub mod telemetry;

pub async fn run(app: Router, addr: SocketAddr) {
    tracing::info!("Starting server in http://{}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[cfg(feature = "shuttle")]
#[shuttle_runtime::main]
pub async fn axum(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_axum::ShuttleAxum {
    let router = routes::create_router(pool).await;
    Ok(router.into())
}
