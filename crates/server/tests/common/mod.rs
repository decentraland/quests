pub mod quest_samples;
pub mod rewards;

use std::time::{SystemTime, UNIX_EPOCH};

use actix_web::body::MessageBody;
use actix_web::dev::ServiceFactory;
use actix_web::web::Data;
use actix_web::App;
use actix_web_lab::__reexports::serde_json;
use dcl_crypto::Identity;
use dcl_http_prom_metrics::HttpMetricsCollectorBuilder;
use quests_db::core::ops::{Connect, GetConnection, Migrate};
use quests_db::{create_quests_db_component, DatabaseOptions, Executor};
use quests_message_broker::messages_queue::RedisMessagesQueue;
use quests_message_broker::redis::Redis;
use quests_server::api::get_app_router;
use quests_server::configuration::Config;
use quests_system::QUESTS_EVENTS_QUEUE_NAME;

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
    let db = create_quests_db_component(&config.database_url, true)
        .await
        .unwrap();

    let redis = Redis::new(&config.redis_url)
        .await
        .expect("> tests > failed to initialize redis");
    let events_queue = RedisMessagesQueue::new(redis.into(), QUESTS_EVENTS_QUEUE_NAME);

    get_app_router(
        &Data::new(config.clone()),
        &Data::new(db),
        &Data::new(events_queue),
        &Data::new(HttpMetricsCollectorBuilder::default().build()),
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

pub fn create_test_identity() -> dcl_crypto::Identity {
    dcl_crypto::Identity::from_json(
      r#"{
     "ephemeralIdentity": {
       "address": "0x84452bbFA4ca14B7828e2F3BBd106A2bD495CD34",
       "publicKey": "0x0420c548d960b06dac035d1daf826472eded46b8b9d123294f1199c56fa235c89f2515158b1e3be0874bfb15b42d1551db8c276787a654d0b8d7b4d4356e70fe42",
       "privateKey": "0xbc453a92d9baeb3d10294cbc1d48ef6738f718fd31b4eb8085efe7b311299399"
     },
     "expiration": "3021-10-16T22:32:29.626Z",
     "authChain": [
       {
         "type": "SIGNER",
         "payload": "0x7949f9f239d1a0816ce5eb364a1f588ae9cc1bf5",
         "signature": ""
       },
       {
         "type": "ECDSA_EPHEMERAL",
         "payload": "Decentraland Login\nEphemeral address: 0x84452bbFA4ca14B7828e2F3BBd106A2bD495CD34\nExpiration: 3021-10-16T22:32:29.626Z",
         "signature": "0x39dd4ddf131ad2435d56c81c994c4417daef5cf5998258027ef8a1401470876a1365a6b79810dc0c4a2e9352befb63a9e4701d67b38007d83ffc4cd2b7a38ad51b"
       }
     ]
    }"#,
  ).unwrap()
}

pub fn get_signed_headers(
    identity: Identity,
    method: &str,
    path: &str,
    metadata: &str,
) -> Vec<(String, String)> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let payload = [method, path, &ts.to_string(), metadata]
        .join(":")
        .to_lowercase();

    let authchain = identity.sign_payload(payload);

    vec![
        (
            "X-Identity-Auth-Chain-0".to_string(),
            serde_json::to_string(authchain.get(0).unwrap()).unwrap(),
        ),
        (
            "X-Identity-Auth-Chain-1".to_string(),
            serde_json::to_string(authchain.get(1).unwrap()).unwrap(),
        ),
        (
            "X-Identity-Auth-Chain-2".to_string(),
            serde_json::to_string(authchain.get(2).unwrap()).unwrap(),
        ),
        ("X-Identity-Timestamp".to_string(), ts.to_string()),
        ("X-Identity-Metadata".to_string(), metadata.to_string()),
    ]
}
