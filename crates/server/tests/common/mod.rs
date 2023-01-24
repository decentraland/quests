use quests_db::core::ops::{Connect, GetConnection, Migrate};
use quests_db::{DatabaseOptions, Executor};
use quests_server::configuration::Config;

pub async fn get_configuration() -> Config {
    let mut config = Config::new().expect("Couldn't read the configuration file");
    let new_url = create_test_db(&config.database_url).await;
    config.database_url = new_url;

    config
}

pub async fn create_test_db(db_url: &str) -> String {
    let split = db_url.split('/');
    let vec = split.collect::<Vec<&str>>();
    let credentials = vec[2].to_string();
    let mut instance_url = format!("postgres://{}", credentials);
    let db_opts = DatabaseOptions::new(&instance_url);
    let db = db_opts.connect().await.unwrap();

    let connection = db.get_conn().await.unwrap();

    let db_name = uuid::Uuid::new_v4().to_string();

    connection
        .detach()
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to create DB");

    instance_url.push_str(format!("/{}", db_name).as_str());

    let pool = DatabaseOptions::new(&instance_url).connect().await.unwrap();

    pool.migrate().await.unwrap();
    instance_url
}
