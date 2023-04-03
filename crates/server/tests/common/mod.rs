pub mod quest_samples;

use actix_web::body::MessageBody;
use actix_web::dev::ServiceFactory;
use actix_web::web::Data;
use actix_web::App;
use dcl_rpc::stream_protocol::GeneratorYielder;
use quests_db::core::ops::{Connect, GetConnection, Migrate};
use quests_db::{create_quests_db_component, DatabaseOptions, Executor};
use quests_definitions::quests::UserUpdate;
use quests_message_broker::init_message_broker_components_with_subscriber;
use quests_server::api::get_app_router;
use quests_server::configuration::Config;

pub async fn get_configuration() -> Config {
    let mut config = Config::new().expect("Couldn't read the configuration file");
    let new_url = create_test_db(&config.database_url).await;
    config.database_url = new_url;

    config
}

pub async fn build_app(
    config: &Config,
) -> App<
    impl ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse<impl MessageBody>,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    let db = create_quests_db_component(&config.database_url)
        .await
        .unwrap();

    let (redis, _) =
        init_message_broker_components_with_subscriber::<GeneratorYielder<UserUpdate>>(
            &config.redis_url,
        )
        .await;

    get_app_router(
        &Data::new(config.clone()),
        &Data::new(db),
        &Data::new(redis),
    )
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
