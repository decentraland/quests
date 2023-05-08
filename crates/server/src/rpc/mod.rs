mod service;
mod warp_ws_transport;

use crate::{auth::authenticate_dcl_user, configuration::Config};
use dcl_crypto::Address;
use dcl_rpc::{server::RpcServer, stream_protocol::GeneratorYielder};
use log::{debug, error};
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
    pub transport_contexts: Arc<RwLock<HashMap<u32, TransportContext>>>,
}

pub struct TransportContext {
    pub subscription: Option<GeneratorYielder<UserUpdate>>,
    pub subscription_handle: Option<JoinHandle<()>>,
    pub user_address: Address,
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
        transport_contexts: Arc::new(RwLock::new(HashMap::new())),
    };

    let transport_contexts: Arc<RwLock<HashMap<u32, TransportContext>>> =
        ctx.transport_contexts.clone();

    let mut rpc_server = RpcServer::create(ctx);

    let rpc_server_events_sender = rpc_server.get_server_events_sender();

    let ws_routes = warp::path::end()
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let server_events_sender = rpc_server_events_sender.clone();
            ws.on_upgrade(|websocket| async move {
                match authenticate_dcl_user(websocket).await {
                    Ok((ws, address)) => {
                        if server_events_sender
                            .send_attach_transport(Arc::new(WarpWebSocketTransport::new(
                                ws, address,
                            )))
                            .is_err()
                        {
                            error!("Couldn't attach web socket transport");
                        }
                    }
                    Err((ws, err)) => {
                        error!("Couldn't authenticate a user {err:?}");
                        let _ = ws.close().await;
                    }
                }
            })
        });

    let health_route = warp::path("health")
        .and(warp::path("live"))
        .and(warp::path::end())
        .map(|| "\"alive\"".to_string());

    let routes = warp::get()
        .and(ws_routes.or(health_route))
        .recover(handle_rejection);

    rpc_server.set_module_registrator_handler(|port| {
        // Registers service for every port
        QuestsServiceRegistration::register_service(port, QuestsServiceImplementation {})
    });

    let cloned_transport_contexts_closes = transport_contexts.clone();
    rpc_server.set_on_transport_closes_handler(move |_, transport_id| {
        let transport_contexts = cloned_transport_contexts_closes.clone();
        tokio::spawn(async move {
            transport_contexts.write().await.remove(&transport_id);
        });
    });

    rpc_server.set_on_transport_connected_handler(move |transport, transport_id| {
        let transport_contexts = transport_contexts.clone();
        tokio::spawn(async move {
            debug!("> OnConnected > Address: {:?}", transport.user_address);
            transport_contexts.write().await.insert(
                transport_id,
                TransportContext {
                    subscription: None,
                    subscription_handle: None,
                    user_address: transport.user_address.clone(),
                },
            );
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
