use anyhow::Context;
use serde::Deserialize;
use std::path::PathBuf;
use std::{env, fs};

#[derive(Deserialize)]
pub struct SSLConfig {
    pub private_key_file: PathBuf,
    pub certificate_chain_file: PathBuf,
}

#[derive(Deserialize)]
pub struct Config {
    pub database_url: String,
    pub signups_enabled: bool,
    pub honeycomb_api_key: String,
    pub ssl: Option<SSLConfig>,
}

impl Config {
    pub fn from_file(path: PathBuf) -> Result<Config, anyhow::Error> {
        let config = fs::read_to_string(path).context("Unable to read config file")?;
        let config: Config =
            toml::from_str(config.as_str()).with_context(|| "Unable to parse config")?;
        Ok(config)
    }

    pub fn from_env() -> Result<Config, anyhow::Error> {
        let signups_enabled = read_env("SIGNUPS_ENABLED")?;
        let signups_enabled = signups_enabled
            .parse()
            .context("Unable to parse SIGNUPS_ENABLED value")?;
        let database_url = read_env("DATABASE_URL")?;
        let honeycomb_api_key = read_env("HONEYCOMB_API_KEY")?;

        let config = Config {
            database_url,
            signups_enabled,
            honeycomb_api_key,
            ssl: None,
        };
        Ok(config)
    }
}

fn read_env(key: &str) -> Result<String, anyhow::Error> {
    env::var(key).with_context(|| format!("Unable to read env var: {}", key))
}
