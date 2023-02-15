use config::ConfigError;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub redis_url: String,
    pub database_url: String,
}

const REDIS_HOST: &str = "REDIS_HOST";
const DATABASE_URL: &str = "DATABASE_URL";

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let config = config::Config::builder()
            .add_source(
                config::Environment::default()
                    .with_list_parse_key(REDIS_HOST)
                    .with_list_parse_key(DATABASE_URL)
                    .try_parsing(true),
            )
            .set_default("redis_url", "quests_redis:6379")?
            .set_default(
                "database_url",
                "postgres://postgres:postgres@quests_db:5432/quests_db",
            )? // => Default for local development
            .build()?;
        config.try_deserialize()
    }
}
