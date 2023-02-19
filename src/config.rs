use serde::Deserialize;
use std::env;
use std::env::VarError;

#[derive(Deserialize)]
pub struct Config {
    pub database_url: String,
    pub signups_enabled: bool,
    pub honeycomb: HoneycombConfig,
}

#[derive(Deserialize)]
pub struct HoneycombConfig {
    pub api_key: String,
    pub dataset: String,
}

#[derive(Debug)]
pub enum ConfigError {
    ConfigNotFound,
    Parse,
}

impl Config {
    pub fn from_env() -> Result<Config, ConfigError> {
        let signups_enabled = read_env("SIGNUPS_ENABLED")?;
        let signups_enabled = match signups_enabled.parse() {
            Ok(s) => s,
            Err(_) => return Err(ConfigError::Parse),
        };
        let database_url = read_env("DATABASE_URL")?;
        let api_key = read_env("HONEYCOMB_API_URL")?;
        let dataset = read_env("HONEYCOMB_DATASET")?;

        let config = Config {
            database_url,
            signups_enabled,
            honeycomb: HoneycombConfig { api_key, dataset },
        };
        Ok(config)
    }
}

fn read_env(key: &str) -> Result<String, ConfigError> {
    match env::var(key) {
        Ok(s) => Ok(s),
        Err(VarError::NotPresent) => Err(ConfigError::ConfigNotFound),
        Err(VarError::NotUnicode(_)) => Err(ConfigError::Parse),
    }
}
