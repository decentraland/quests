use config::{self, ConfigError};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server_port: u16,
    pub database_url: String, // Using the URL directly has benefits for SQLX macros and it can be built in deploy time
    pub env: String,
    pub wkc_metrics_bearer_token: String,
}

const METRICS_TOKEN: &str = "WKC_METRICS_BEARER_TOKEN"; // WCK ENV
const ENV_VAR: &str = "ENV";
const DATABASE_URL: &str = "DATABASE_URL";

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
                    .with_list_parse_key(DATABASE_URL)
                    .try_parsing(true),
            )
            .set_default("server_port", 8080)? // It should be empty for local development
            .set_default("env", "dev")?
            .set_default("wkc_metrics_bearer_token", "")?
            .set_default(
                "database_url",
                "postgres://postgres:postgres@quests_db:5432/quests_db",
            )? // => Default for local development
            .build()?;

        config.try_deserialize()
    }
}
