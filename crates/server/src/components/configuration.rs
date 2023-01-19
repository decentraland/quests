use config::{self, ConfigError};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: Server,
    pub env: String,
    pub wkc_metrics_bearer_token: String,
}

const METRICS_TOKEN: &str = "WKC_METRICS_BEARER_TOKEN"; // WCK ENV
const ENV_VAR: &str = "ENV";

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let config = config::Config::builder()
            .add_source(
                config::Environment::with_prefix("HTTP") // => For HTTP_SERVER_PORT in WCK ENV
                    .separator("_")
                    .list_separator(" ")
                    .try_parsing(true),
            )
            .add_source(
                config::Environment::default()
                    .with_list_parse_key(METRICS_TOKEN)
                    .with_list_parse_key(ENV_VAR)
                    .try_parsing(true),
            )
            .set_default("server.host", "0.0.0.0")? // We use the default because it should not often change
            .set_default("server.port", 8080)? // It should be empty for local development
            .set_default("env", "dev")?
            .set_default("wkc_metrics_bearer_token", "")?
            .build()?;

        config.try_deserialize()
    }
}
