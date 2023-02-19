use serde::Deserialize;

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
