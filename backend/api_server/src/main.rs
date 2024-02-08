use std::env;
use api_server::routes;
use once_cell::sync::Lazy;
use utils::configuration::Settings;
use utils::state::AppState;

static TRACING: Lazy<()> = Lazy::new(|| {
    let subscriber = api_server::telemetry::get_subscriber();
    api_server::telemetry::init_subscriber(subscriber);
});

#[tokio::main]
async fn main() {
    let configuration = Settings::new().expect("Failed to read configuration");

    let args: Vec<String> = env::args().collect();
    if args.contains(&"migrate".to_string()){
        println!("Migrating the database");
        let app_state = AppState::init(configuration).await;
        api_server::migrate::migrate_all_apps(&app_state.connection).await;
        return;
    }

    let addr = configuration.application.url().parse().unwrap();

    Lazy::force(&TRACING);

    let app = routes::create_router().await;
    api_server::run(app, addr).await;
}
