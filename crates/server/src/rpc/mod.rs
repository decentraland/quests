mod service;
mod warp_ws_transport;

use crate::configuration::Config;
use dcl_rpc::{server::RpcServer, stream_protocol::GeneratorYielder};
use quests_db::Database;
use quests_message_broker::{
    channel::{ChannelSubscriber, RedisChannelSubscriber},
    messages_queue::RedisMessagesQueue,
    QUEST_UPDATES_CHANNEL_NAME,
};
use quests_protocol::quests::{user_update::Message, QuestsServiceRegistration, UserUpdate};
use service::QuestsServiceImplementation;
use std::{collections::HashMap, sync::Arc};
use tokio::{sync::RwLock, task::JoinHandle};
use warp::{
    http::StatusCode,
    reject::{MissingHeader, Reject},
    reply, Filter, Rejection, Reply,
};
use warp_ws_transport::WarpWebSocketTransport;

pub struct QuestsRpcServerContext {
    pub config: Arc<Config>,
    pub db: Arc<Database>,
    pub redis_events_queue: Arc<RedisMessagesQueue>,
    pub quest_subscriptions: Arc<RwLock<HashMap<String, GeneratorYielder<UserUpdate>>>>,
}

pub async fn run_rpc_server(
    (config, db, redis_events_queue, redis_quests_channel_subscriber): (
        Arc<Config>,
        Arc<Database>,
        Arc<RedisMessagesQueue>,
        RedisChannelSubscriber,
    ),
) -> (JoinHandle<()>, JoinHandle<()>) {
    let ws_server_address = ([0, 0, 0, 0], config.ws_server_port.parse::<u16>().unwrap());
    let ctx = QuestsRpcServerContext {
        config,
        db,
        redis_events_queue,
        quest_subscriptions: Arc::new(RwLock::new(HashMap::new())),
    };

    let subscriptions = ctx.quest_subscriptions.clone();

    redis_quests_channel_subscriber.subscribe(
        QUEST_UPDATES_CHANNEL_NAME,
        move |user_update: UserUpdate| {
            let subscriptions = subscriptions.clone();
            async move {
                if let Some(Message::QuestState(state)) = &user_update.message {
                    let subs = subscriptions.read().await;
                    if let Some(generator) = subs.get(&state.quest_instance_id) {
                        generator
                            .r#yield(user_update)
                            .await
                            .expect("to be able to send the update"); // todo: handle error
                    }
                }
            }
        },
    );

    let mut rpc_server = RpcServer::create(ctx);

    let rpc_server_events_sender = rpc_server.get_server_events_sender();

    let routes = warp::path::end()
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let server_events_sender = rpc_server_events_sender.clone();
            ws.on_upgrade(|websocket| async move {
                server_events_sender
                    .send_attach_transport(Arc::new(WarpWebSocketTransport::new(websocket)))
                    .unwrap();
            })
        })
        .recover(handle_rejection);

    rpc_server.set_handler(|port| {
        // Registers service for every port
        QuestsServiceRegistration::register_service(port, QuestsServiceImplementation {})
    });

    let rpc_server_handle = tokio::spawn(async move {
        rpc_server.run().await;
    });

    let http_server_handle = tokio::spawn(async move {
        warp::serve(routes).run(ws_server_address).await;
    });

    (http_server_handle, rpc_server_handle)
}

#[derive(Debug)]
struct Unauthorized {}

impl Reject for Unauthorized {}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
    if err.find::<Unauthorized>().is_some() {
        Ok(reply::with_status("UNAUTHORIZED", StatusCode::UNAUTHORIZED))
    } else if err.find::<MissingHeader>().is_some() {
        Ok(reply::with_status("BAD_REQUEST", StatusCode::BAD_REQUEST))
    } else {
        eprintln!("unhandled rejection: {:?}", err);
        Ok(reply::with_status(
            "INTERNAL_SERVER_ERROR",
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}
