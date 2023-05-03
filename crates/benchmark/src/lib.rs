use futures_util::StreamExt;
use log::{debug, info};
use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use dcl_rpc::{client::RpcClient, transports::web_socket::WebSocketTransport};
use futures_util::{lock::Mutex, stream::FuturesUnordered};
use rand::{seq::IteratorRandom, thread_rng};

include!(concat!(env!("OUT_DIR"), "/decentraland.quests.rs"));

pub const SERVER_HTTP: &str = "http://0.0.0.0:3000";
pub const SERVER_WS: &str = "ws://0.0.0.0:3001";

pub mod quests;
pub mod simulation;

#[async_trait]
pub trait Context {
    async fn init() -> Self;
}

#[async_trait]
pub trait Client<C: Context> {
    async fn from_rpc_client(client: RpcClient<WebSocketTransport>) -> Self;
    async fn act(self, context: &C) -> Self;
}

pub struct Simulation;

impl Simulation {
    pub async fn run<SC, C>(rpc_clients: Vec<RpcClient<WebSocketTransport>>)
    where
        SC: Context + Send + Sync + 'static,
        C: Client<SC> + Send + Sync + 'static,
    {
        let context = SC::init().await;
        let mut clients = vec![];
        for rpc_client in rpc_clients {
            clients.push(C::from_rpc_client(rpc_client).await);
        }

        let clients = Arc::new(Mutex::new(clients));
        let context = Arc::new(context);

        let test_duration = Duration::from_secs(60 * 5);

        debug!("Simulation > Wait for 10s before start...");
        sleep(Duration::from_secs(10));
        let mut futures = FuturesUnordered::new();
        for worker_id in 0..100 {
            futures.push(tokio::spawn(worker(
                worker_id,
                test_duration,
                clients.clone(),
                context.clone(),
            )));
        }

        while futures.next().await.is_some() {}
    }
}

async fn worker<SC, C>(
    worker_id: usize,
    duration: Duration,
    clients: Arc<Mutex<Vec<C>>>,
    context: Arc<SC>,
) where
    SC: Context + Send + Sync,
    C: Client<SC> + Send + Sync,
{
    let start = Instant::now();
    loop {
        if start.elapsed() > duration {
            break;
        }
        debug!("Worker {worker_id} > Locking clients");
        let mut clients_guard = clients.lock().await;
        let i = (0..clients_guard.len()).choose(&mut thread_rng());
        if let Some(i) = i {
            let client = clients_guard.remove(i);
            drop(clients_guard);
            debug!("Worker {worker_id} > Clients guard manually dropped");

            let client = client.act(&context).await;
            debug!("Worker {worker_id} > client {i} acted");

            debug!("Worker {worker_id} > Locking clients");
            clients.lock().await.push(client);
            debug!("Worker {worker_id} > Unlocking clients");
        } else {
            debug!("Worker {worker_id} > Clients guard automatic drop");
        }

        let millis = 100;
        debug!("Worker {worker_id} > Waiting {millis} ms before next iteration");
        sleep(Duration::from_millis(millis));
    }
    info!("Worker {worker_id} > Returning");
}
