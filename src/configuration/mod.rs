use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Debug, Deserialize)]
pub struct ObjectConfig {
    pub app_name: Option<String>,
    pub database: String,
    pub collection: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiConfig {
    pub object: ObjectConfig,
}

#[derive(Debug, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub api: ApiConfig,
    pub metrics: MetricsConfig,
}

impl Config {
    pub fn parse_config(path: impl AsRef<Path>) -> anyhow::Result<Config> {
        let raw = fs::read_to_string(path)?;
        Ok(serde_json::from_str::<Config>(&raw)?)
    }
}
