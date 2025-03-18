// config/mod.rs
use serde::Deserialize;
use config::Config;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub metrics: MetricsSettings,
}

#[derive(Debug, Deserialize)]
pub struct ServerSettings {
    pub address: String,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize)]
pub struct MetricsSettings {
    pub enabled: bool,
    pub port: u16,
}

impl Settings {
    pub fn new() -> Result<Self, config::ConfigError> {
        let settings = Config::builder()
            .add_source(config::File::with_name("config/config"))
            .add_source(config::Environment::with_prefix("APP"))
            .build()?;

        settings.try_deserialize()
    }
}