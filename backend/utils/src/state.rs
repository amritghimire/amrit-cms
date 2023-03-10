use crate::configuration::{RunMode, Settings};
use crate::email::EmailClient;
use axum::extract::FromRef;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
    pub connection: PgPool,
    pub email_client: EmailClient,
}

impl AppState {
    pub async fn init(settings: Settings) -> Self {
        let connection = PgPoolOptions::new()
            .acquire_timeout(Duration::from_secs(2))
            .connect_lazy(&settings.database.connection_string())
            .expect("Failed to connect to Postgres.");
        let email_client = EmailClient::init(settings.email.clone());

        Self {
            settings,
            connection,
            email_client,
        }
    }

    pub fn from_conn(connection: PgPool) -> Self {
        let settings = Settings::new().expect("Unable to fetch config");
        let email_client = EmailClient::init(settings.email.clone());

        Self {
            settings,
            connection,
            email_client,
        }
    }

    pub fn test_state(connection: PgPool) -> Self {
        let settings = Settings::get_config(RunMode::Test).expect("Unable to fetch test config");
        let email_client = EmailClient::init(settings.email.clone());

        Self {
            settings,
            connection,
            email_client,
        }
    }
}

impl FromRef<AppState> for PgPool {
    fn from_ref(app_state: &AppState) -> PgPool {
        app_state.connection.clone()
    }
}

impl FromRef<AppState> for Settings {
    fn from_ref(app_state: &AppState) -> Settings {
        app_state.settings.clone()
    }
}

impl FromRef<AppState> for EmailClient {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.email_client.clone()
    }
}
