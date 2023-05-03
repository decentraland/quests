mod service;
mod warp_ws_transport;

use crate::configuration::Config;
use dcl_rpc::{server::RpcServer, stream_protocol::GeneratorYielder};
use log::error;
use quests_db::Database;
use quests_message_broker::{channel::RedisChannelSubscriber, messages_queue::RedisMessagesQueue};
use quests_protocol::quests::{QuestsServiceRegistration, UserUpdate};
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
    pub redis_channel_subscriber: RedisChannelSubscriber,
    pub transport_contexts: TransportContext,
}

#[derive(Default)]
pub struct TransportContext {
    pub subscriptions: Arc<RwLock<HashMap<u32, GeneratorYielder<UserUpdate>>>>,
    pub subscriptions_handle: Arc<RwLock<HashMap<u32, JoinHandle<()>>>>,
}

pub async fn run_rpc_server(
    (config, db, redis_events_queue, redis_channel_subscriber): (
        Arc<Config>,
        Arc<Database>,
        Arc<RedisMessagesQueue>,
        RedisChannelSubscriber,
    ),
) -> (JoinHandle<()>, JoinHandle<()>) {
    let ws_server_address = (
        [0, 0, 0, 0],
        config.ws_server_port.parse::<u16>().unwrap_or(5001),
    );
    let ctx = QuestsRpcServerContext {
        config,
        db,
        redis_events_queue,
        redis_channel_subscriber,
        transport_contexts: TransportContext::default(),
    };

    let subscription_handles_cloned = ctx.transport_contexts.subscriptions_handle.clone();
    let subscriptions_cloned = ctx.transport_contexts.subscriptions.clone();

    let mut rpc_server = RpcServer::create(ctx);

    let rpc_server_events_sender = rpc_server.get_server_events_sender();

    let routes = warp::path::end()
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let server_events_sender = rpc_server_events_sender.clone();
            ws.on_upgrade(|websocket| async move {
                if server_events_sender
                    .send_attach_transport(Arc::new(WarpWebSocketTransport::new(websocket)))
                    .is_err()
                {
                    error!("Couldn't attach web socket transport");
                }
            })
        })
        .recover(handle_rejection);

    rpc_server.set_module_registrator_handler(|port| {
        // Registers service for every port
        QuestsServiceRegistration::register_service(port, QuestsServiceImplementation {})
    });

    rpc_server.set_on_transport_closes_handler(move |_, transport_id| {
        let subs_join_handle = subscription_handles_cloned.clone();
        let subs_generator = subscriptions_cloned.clone();
        tokio::spawn(async move {
            subs_join_handle.write().await.remove(&transport_id);
            subs_generator.write().await.remove(&transport_id)
        });
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
