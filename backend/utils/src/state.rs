use crate::configuration::{RunMode, Settings};
use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use email_clients::clients::{get_email_client, EmailClient};
use email_clients::configuration::EmailConfiguration;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::mpsc::SyncSender;
use std::time::Duration;
use tokio::task::JoinHandle;
use uuid::Uuid;

pub struct BackgroundTask {
    pub name: String,
    pub handle: JoinHandle<()>,
    pub identifier: Uuid,
}

impl BackgroundTask {
    pub fn new(name: impl AsRef<str>, handle: JoinHandle<()>) -> Self {
        Self {
            name: name.as_ref().to_string(),
            handle,
            identifier: Uuid::new_v4(),
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
    pub connection: PgPool,
    pub email_client: EmailClient,
    pub tasks: Option<SyncSender<BackgroundTask>>,
}

impl AppState {
    pub async fn init(settings: Settings) -> Self {
        let connection = PgPoolOptions::new()
            .acquire_timeout(Duration::from_secs(2))
            .connect_lazy(&settings.database.connection_string())
            .expect("Failed to connect to Postgres.");

        let email_configuration: EmailConfiguration = settings
            .clone()
            .email
            .try_into()
            .expect("Invalid email configuration");
        let email_client = get_email_client(email_configuration);

        Self {
            settings,
            connection,
            email_client,
            tasks: None,
        }
    }

    pub fn from_conn(connection: PgPool) -> Self {
        let settings = Settings::new().expect("Unable to fetch config");
        let email_configuration: EmailConfiguration = settings
            .clone()
            .email
            .try_into()
            .expect("Invalid email configuration");
        let email_client = get_email_client(email_configuration);

        Self {
            settings,
            connection,
            email_client,
            tasks: None,
        }
    }

    pub fn test_state(connection: PgPool, config: Option<Settings>) -> Self {
        let settings = config.unwrap_or_else(|| {
            Settings::get_config(RunMode::Test).expect("Unable to fetch test config")
        });
        let email_configuration: EmailConfiguration = settings
            .clone()
            .email
            .try_into()
            .expect("Invalid email configuration");
        let email_client = get_email_client(email_configuration);

        Self {
            settings,
            connection,
            email_client,
            tasks: None,
        }
    }

    pub fn test_email_state(connection: PgPool, email_client: EmailClient) -> Self {
        let settings = Settings::get_config(RunMode::Test).expect("Unable to fetch test config");

        Self {
            settings,
            connection,
            email_client,
            tasks: None,
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
    fn from_ref(app_state: &AppState) -> EmailClient {
        app_state.email_client.clone()
    }
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        Key::from(state.settings.application.key.as_ref())
    }
}
