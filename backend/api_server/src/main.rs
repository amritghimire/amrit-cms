#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:3000".parse().unwrap();
    api_server::run(addr).await;
}
