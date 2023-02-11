use config::{Config, ConfigError, Environment, File};
use std::env;

#[derive(Debug, serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16
}

#[derive(Debug, serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }
}



impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        Self::get_config(&run_mode)
    }

    pub fn get_config(run_mode: &str) -> Result<Self, ConfigError> {
        let mut dir = env::current_dir().unwrap();
        let mut default_config = dir.join("config").join("default.yaml");
        while !default_config.exists() {
            dir = dir.parent().unwrap().into();
            default_config = dir.join("config").join("default.yaml");
        }

        let s = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::from(dir.join("config/default.yaml")))
            // Add in the current environment file
            // Default to 'development' env
            // Note that this file is _optional_
            .add_source(
                File::from(dir.join(format!("config/{}", run_mode)))
                    .required(false),
            )
            // Add in a local configuration file
            // This file shouldn't be checked in to git
            .add_source(File::from(dir.join("config/local")).required(false))
            // Add in settings from the environment (with a prefix of APP)
            // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
            .add_source(Environment::with_prefix("app"))
            // You may also programmatically change settings
            .build()?;
        // You can deserialize (and thus freeze) the entire configuration as
        s.try_deserialize()
    }
}