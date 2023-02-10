use std::net::SocketAddr;
use axum::Router;
use axum::routing::IntoMakeService;
use hyper::Server;
use hyper::server::conn::AddrIncoming;

mod handlers;
pub mod routes;

pub fn run(app: Router, addr: SocketAddr) -> Server<AddrIncoming, IntoMakeService<Router>> {

    println!("Starting server in {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
}
