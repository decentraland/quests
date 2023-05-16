use quests_db::{
    core::ops::{Connect, GetConnection, Migrate},
    DatabaseOptions, Executor,
};
use quests_system::configuration::Config;

pub async fn create_test_db() -> String {
    let config = Config::new().expect("can read config");
    let split = config.database_url.split('/');
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
