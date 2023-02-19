use axum::routing::IntoMakeService;
use axum::Router;
use hyper::server::conn::AddrIncoming;
use hyper::Server;
use std::net::SocketAddr;

mod handlers;
pub mod routes;
pub mod telemetry;

pub fn run(app: Router, addr: SocketAddr) -> Server<AddrIncoming, IntoMakeService<Router>> {
    println!("Starting server in {}", addr);

    axum::Server::bind(&addr).serve(app.into_make_service())
}
