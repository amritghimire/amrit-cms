use api_server::routes;
use utils::configuration::Settings;

#[tokio::main]
async fn main() {
    let configuration = Settings::new().expect("Failed to read configuration");
    let addr = format!("0.0.0.0:{}", configuration.application_port)
        .parse()
        .unwrap();
    let app = routes::create_router().await;
    api_server::run(app, addr).await.unwrap();
}
