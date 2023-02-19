use config::{Config, ConfigError, Environment, File};
use std::env;
use std::path::PathBuf;
use secrecy::{ExposeSecret, Secret};

#[derive(Debug, PartialEq, Eq)]
pub enum RunMode {
    Development,
    Production,
    Test,
    Local
}

impl RunMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            RunMode::Test => "test",
            RunMode::Production => "production",
            RunMode::Development => "development",
            RunMode::Local => "local"
        }
    }
}

impl TryFrom<String> for RunMode {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            "development" => Ok(Self::Development),
            "test" => Ok(Self::Test),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other
            )),
        }
    }
}


#[derive(Clone, Debug, serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password.expose_secret(), self.host, self.port, self.database_name
        )
    }
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        // Detect the running environment.
        // Default to `local` if unspecified.
        let run_mode: RunMode = env::var("RUN_MODE")
            .unwrap_or_else(|_| "development".into())
            .try_into()
            .expect("Failed to parse APP_ENVIRONMENT.");
        Self::get_config(run_mode)
    }

    pub fn get_config(run_mode: RunMode) -> Result<Self, ConfigError> {
        let dir = Self::get_root_dir();

        let mut s = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::from(dir.join("config/default.yaml")));

        if run_mode == RunMode::Test {
            // Add in a local configuration file
            // This file shouldn't be checked in to git
            s = s
                .add_source(File::from(dir.join("config/local")).required(false))
                .add_source(File::from(dir.join(format!("config/{}", run_mode.as_str()))).required(false))
        } else {
            s = s
                .add_source(File::from(dir.join(format!("config/{}", run_mode.as_str()))).required(false))
                .add_source(File::from(dir.join("config/local")).required(false))
        }
        let s = s
            // Add in settings from the environment (with a prefix of APP)
            // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
            .add_source(Environment::with_prefix("app"))
            // You may also programmatically change settings
            .build()?;
        // You can deserialize (and thus freeze) the entire configuration as
        s.try_deserialize()
    }

    fn get_root_dir() -> PathBuf {
        let mut dir = env::current_dir().unwrap();
        let mut default_config = dir.join("config").join("default.yaml");
        while !default_config.exists() {
            dir = dir.parent().unwrap().into();
            default_config = dir.join("config").join("default.yaml");
        }
        dir
    }
}
