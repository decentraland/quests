use quests_db_sqlx::Database;

use super::{database::create_db_component, Config};

pub struct AppComponents {
    pub config: Config,
    pub database: Database,
}

impl AppComponents {
    pub async fn new(custom_config: Option<Config>) -> Self {
        let config = custom_config
            .unwrap_or_else(|| Config::new().expect("unable to build up the App configuration"));

        let database = create_db_component(&config.database_url).await;

        Self { config, database }
    }
}
