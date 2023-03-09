use config::ConfigError;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub redis_url: String,
    pub database_url: String,
}

const REDIS_URL: &str = "REDIS_URL";
const DATABASE_URL: &str = "DATABASE_URL";

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let config = config::Config::builder()
            .add_source(
                config::Environment::default()
                    .with_list_parse_key(REDIS_URL)
                    .with_list_parse_key(DATABASE_URL)
                    .try_parsing(true),
            )
            .set_default("redis_url", "127.0.0.1:6379")?
            .set_default(
                "database_url",
                "postgres://postgres:postgres@127.0.0.1:5432/quests_db",
            )? // => Default for local development
            .build()?;
        config.try_deserialize()
    }
}
