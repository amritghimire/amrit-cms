use once_cell::sync::Lazy;
use api_server::routes;
use utils::configuration::Settings;


static TRACING: Lazy<()> = Lazy::new(|| {
    let subscriber = api_server::telemetry::get_subscriber();
    api_server::telemetry::init_subscriber(subscriber);
});


#[tokio::main]
async fn main() {
    let configuration = Settings::new().expect("Failed to read configuration");
    let addr = format!("0.0.0.0:{}", configuration.application_port)
        .parse()
        .unwrap();

    Lazy::force(&TRACING);

    let app = routes::create_router().await;
    api_server::run(app, addr).await.unwrap();
}
