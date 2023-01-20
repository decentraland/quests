use quests_db_core::ops::Connect;
use quests_db_sqlx::{Database, DatabaseOptions};

pub async fn create_db_component(db_url: &str) -> Database {
    let mut db_options = DatabaseOptions::new(db_url);
    db_options.pool_options = db_options
        .pool_options
        .min_connections(5)
        .max_connections(10);

    let db_pool = db_options.connect().await;

    match db_pool {
        Ok(db) => db,
        Err(error) => {
            log::error!("> Database > Error while connecting to DB {error:?}");
            panic!("{error:?}");
        }
    }
}
