use api_server::routes;

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:3000".parse().unwrap();
    let app = routes::create_router();
    api_server::run(app, addr).await.unwrap();
}
