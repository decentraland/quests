use config::{self, ConfigError};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database_url: String, // Using the URL directly has benefits for SQLX macros and it can be built in deploy time
    pub redis_url: String,
    pub env: String,
    pub wkc_metrics_bearer_token: String,
    pub http_port: u16,
    pub ws_port: String,
}

const METRICS_TOKEN: &str = "WKC_METRICS_BEARER_TOKEN"; // WCK ENV
const ENV_VAR: &str = "ENV";
const DATABASE_URL: &str = "DATABASE_URL";
const REDIS_URL: &str = "REDIS_URL";
const HTTP_PORT: &str = "HTTP_PORT";
const WS_PORT: &str = "WS_PORT"; // Server port for the Websocket server used by the RpcServer

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
                    .with_list_parse_key(REDIS_URL)
                    .with_list_parse_key(HTTP_PORT)
                    .with_list_parse_key(WS_PORT)
                    .try_parsing(true),
            )
            .set_default("http_port", 5000)? // It's empty for local development
            .set_default("ws_port", 5001)? // default for local development
            .set_default("env", "dev")?
            .set_default("wkc_metrics_bearer_token", "")?
            .set_default(
                "database_url",
                "postgres://postgres:postgres@localhost:5432/quests_db",
            )? // => Default for local development
            .set_default("redis_url", "localhost:6379")?
            .build()?;

        config.try_deserialize()
    }
}
