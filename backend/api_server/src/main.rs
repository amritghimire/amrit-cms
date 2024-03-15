use std::env;

#[cfg(feature = "local")]
use api_server::routes;
#[cfg(feature = "local")]
use once_cell::sync::Lazy;
#[cfg(feature = "local")]
use utils::configuration::Settings;
#[cfg(feature = "local")]
use utils::state::AppState;

#[cfg(feature = "shuttle")]
use api_server::routes;
#[cfg(feature = "shuttle")]
use shuttle_secrets::SecretStore;
#[cfg(feature = "shuttle")]
use sqlx::PgPool;

#[cfg(feature = "local")]
static TRACING: Lazy<()> = Lazy::new(|| {
    let subscriber = api_server::telemetry::get_subscriber();
    api_server::telemetry::init_subscriber(subscriber);
});

#[cfg(feature = "local")]
#[tokio::main]
async fn main() {
    let configuration = Settings::new().expect("Failed to read configuration");
    let addr = configuration.application.url().parse().unwrap();
    let app_state = AppState::init(configuration).await;

    let args: Vec<String> = env::args().collect();
    if args.contains(&"migrate".to_string()) {
        println!("Migrating the database");
        api_server::migrate::migrate_all_apps(&app_state.connection).await;
        return;
    }

    Lazy::force(&TRACING);

    let app = routes::create_router(app_state.connection).await;
    api_server::run(app, addr).await;
}

#[cfg(feature = "shuttle")]
#[shuttle_runtime::main]
pub async fn axum(
    #[shuttle_shared_db::Postgres] pool: PgPool,
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_axum::ShuttleAxum {
    tracing::info!(
        "RUN ENV IS {}",
        secret_store.get("RUN_ENV").unwrap_or("NONE".to_string())
    );
    set_secrets_to_env(secret_store);
    let router = routes::create_router(pool).await;
    Ok(router.into())
}

#[cfg(feature = "shuttle")]
pub fn set_secrets_to_env(secret_store: SecretStore) {
    tracing::info!("Starting secrets");

    for (key, value) in secret_store {
        env::set_var(key, value);
    }
}
