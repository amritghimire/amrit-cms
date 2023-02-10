use std::net::SocketAddr;

mod handlers;
pub mod routes;

pub async fn run(addr: SocketAddr) {
    let app = routes::create_router();

    println!("Starting server in http://0.0.0.0:3000/ ");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
