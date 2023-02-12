use crate::configuration::Settings;
use axum::extract::FromRef;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
    pub connection: PgPool,
}

impl AppState {
    pub async fn init(settings: Settings) -> Self {
        let connection = PgPool::connect(&settings.database.connection_string())
            .await
            .expect("Failed to connect to Postgres.");
        Self {
            settings,
            connection,
        }
    }

    pub fn from_conn(connection: PgPool) -> Self {
        let settings = Settings::new().expect("Unable to fetch config");
        Self {
            settings,
            connection,
        }
    }

    pub fn test_state(connection: PgPool) -> Self {
        let settings = Settings::get_config("test").expect("Unable to fetch test config");
        Self {
            settings,
            connection,
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
