mod metrics_collector;
mod service;

use crate::configuration::Config;
use async_trait::async_trait;
use dcl_crypto::{Address, Authenticator};
use dcl_crypto_middleware_rs::ws_signed_headers::{
    authenticate_dcl_user_with_signed_headers, AuthenticatedWSWithSignedHeaders,
};
use dcl_rpc::{
    server::RpcServer,
    transports::web_sockets::{warp::WarpWebSocket, Message, WebSocket, WebSocketTransport},
};
use futures_util::lock::Mutex;
use log::{debug, error, info};
use quests_db::Database;
use quests_message_broker::{
    channel::{RedisChannelPublisher, RedisChannelSubscriber},
    messages_queue::RedisMessagesQueue,
};
use quests_protocol::definitions::*;
use service::QuestsServiceImplementation;
use std::{collections::HashMap, sync::Arc};
use tokio::{sync::RwLock, task::JoinHandle, time::Instant};
use warp::{
    http::{HeaderValue, StatusCode},
    reject::{MissingHeader, Reject},
    reply, Filter, Rejection, Reply,
};

use self::metrics_collector::MetricsCollector;

pub struct QuestsRpcServerContext {
    pub config: Arc<Config>,
    pub db: Arc<Database>,
    pub redis_events_queue: Arc<RedisMessagesQueue>,
    pub redis_channel_subscriber: RedisChannelSubscriber,
    pub redis_channel_publisher: Arc<RedisChannelPublisher>,
    pub transport_contexts: Arc<RwLock<HashMap<u32, TransportContext>>>,
    pub metrics_collector: Arc<MetricsCollector>,
}

pub struct TransportContext {
    pub subscription_handle: Option<(JoinHandle<()>, Instant)>,
    pub quest_instance_ids: Arc<Mutex<Vec<String>>>,
    pub user_address: Address,
    pub connection_ts: Instant,
}

pub async fn run_rpc_server(
    (config, db, redis_events_queue, redis_channel_subscriber, redis_channel_publisher): (
        Arc<Config>,
        Arc<Database>,
        Arc<RedisMessagesQueue>,
        RedisChannelSubscriber,
        Arc<RedisChannelPublisher>,
    ),
) -> (JoinHandle<()>, JoinHandle<()>) {
    let ws_server_address = (
        [0, 0, 0, 0],
        config.ws_server_port.parse::<u16>().unwrap_or(5001),
    );

    let metrics_collector = Arc::new(MetricsCollector::new());
    let metrics_token = config.wkc_metrics_bearer_token.clone();

    let ctx = QuestsRpcServerContext {
        config,
        db,
        redis_events_queue,
        redis_channel_subscriber,
        redis_channel_publisher,
        transport_contexts: Arc::new(RwLock::new(HashMap::new())),
        metrics_collector: metrics_collector.clone(),
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
                let websocket = WarpWebSocket::new(websocket);
                let Ok(address) = authenticate_dcl_user_with_signed_headers(
                    "get",
                    "/",
                    &mut AuthWs(&websocket),
                    30,
                    Authenticator::new(),
                )
                .await
                else {
                    debug!("Couldn't authenticate a user, closing connection...");
                    let _ = websocket.close().await;
                    return;
                };

                debug!("> User connected: {address:?}");

                let websocket = Arc::new(websocket);
                ping_every_30s(websocket.clone());
                let transport = Arc::new(WebSocketTransport::with_context(websocket, address));

                if server_events_sender
                    .send_attach_transport(transport)
                    .is_err()
                {
                    error!("Couldn't attach web socket transport");
                }
            })
        });

    let health_route = warp::path("health")
        .and(warp::path("live"))
        .and(warp::path::end())
        .map(|| "\"alive\"".to_string());

    let metrics_collector_cloned = Arc::clone(&metrics_collector);
    let metrics_route = warp::get()
        .and(warp::path("metrics"))
        .and(warp::path::end())
        .and(warp::header::value("Authorization"))
        .and_then(move |header_value: HeaderValue| {
            let expected_token = metrics_token.clone();
            validate_bearer_token(header_value, expected_token)
        })
        .untuple_one()
        .and(warp::any().map(move || Arc::clone(&metrics_collector_cloned)))
        .and_then(|metrics_collector: Arc<MetricsCollector>| async move {
            if let Ok(metrics) = metrics_collector.collect() {
                Ok(metrics)
            } else {
                Err(warp::reject())
            }
        });

    let routes = warp::get()
        .and(ws_routes.or(health_route).or(metrics_route))
        .recover(handle_rejection);

    rpc_server.set_module_registrator_handler(|port| {
        QuestsServiceRegistration::register_service(port, QuestsServiceImplementation {})
    });

    let cloned_transport_contexts_closes = transport_contexts.clone();
    let metrics_collector_cloned = Arc::clone(&metrics_collector);
    rpc_server.set_on_transport_closes_handler(move |_, transport_id| {
        let transport_contexts = cloned_transport_contexts_closes.clone();
        metrics_collector_cloned.client_disconnected();

        let metrics_collector = metrics_collector_cloned.clone();

        tokio::spawn(async move {
            let transport = transport_contexts.write().await.remove(&transport_id);
            if let Some(transport) = transport {
                metrics_collector
                    .record_client_duration(transport.connection_ts.elapsed().as_secs_f64());
                if let Some((_, instant)) = transport.subscription_handle {
                    metrics_collector.record_subscribe_duration(instant.elapsed().as_secs_f64())
                }
            }
        });
    });

    rpc_server.set_on_transport_connected_handler(move |transport, transport_id| {
        let transport_contexts = transport_contexts.clone();
        metrics_collector.client_connected();
        tokio::spawn(async move {
            debug!("> OnConnected > Address: {:?}", transport.context);
            transport_contexts.write().await.insert(
                transport_id,
                TransportContext {
                    subscription_handle: None,
                    quest_instance_ids: Arc::new(Mutex::new(vec![])),
                    user_address: transport.context,
                    connection_ts: Instant::now(),
                },
            );
        });
    });

    let rpc_server_handle = tokio::spawn(async move {
        rpc_server.run().await;
    });

    let http_server_handle = tokio::spawn(async move {
        info!("Quests WS Server running at: {:?}", ws_server_address);
        warp::serve(routes).run(ws_server_address).await;
    });

    (http_server_handle, rpc_server_handle)
}

fn ping_every_30s(websocket: Arc<WarpWebSocket>) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            if websocket.send(Message::Ping).await.is_err() {
                break;
            }
        }
    });
}

#[derive(Debug)]
struct Unauthorized;
impl Reject for Unauthorized {}

#[derive(Debug)]
struct InvalidHeader;
impl Reject for InvalidHeader {}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
    if err.find::<Unauthorized>().is_some() {
        Ok(reply::with_status("UNAUTHORIZED", StatusCode::UNAUTHORIZED))
    } else if err.find::<InvalidHeader>().is_some() || err.find::<MissingHeader>().is_some() {
        Ok(reply::with_status("BAD_REQUEST", StatusCode::BAD_REQUEST))
    } else {
        Ok(reply::with_status(
            "INTERNAL_SERVER_ERROR",
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}

struct AuthWs<'a>(&'a WarpWebSocket);
#[async_trait]
impl<'a> AuthenticatedWSWithSignedHeaders for AuthWs<'a> {
    type Error = ();

    async fn receive_signed_headers(&mut self) -> Result<String, Self::Error> {
        match self.0.receive().await {
            Some(Ok(Message::Text(text_reply))) => Ok(text_reply),
            Some(_) => Err(()),
            None => Err(()),
        }
    }
}

pub async fn validate_bearer_token(
    header_value: HeaderValue,
    expected_token: String,
) -> Result<(), Rejection> {
    header_value
        .to_str()
        .map_err(|_| warp::reject::custom(InvalidHeader))
        .and_then(|header_value_str| {
            let split_header_bearer = header_value_str.split(' ').collect::<Vec<&str>>();
            let token = split_header_bearer.get(1);
            let token = token.map_or("", |token| token.to_owned());

            if token == expected_token {
                Ok(())
            } else {
                Err(warp::reject::custom(Unauthorized))
            }
        })
}
