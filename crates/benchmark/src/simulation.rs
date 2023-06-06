use std::{
    collections::HashSet,
    sync::Arc,
    thread::sleep,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use dcl_rpc::client::RpcClient;
use futures_util::{stream::FuturesUnordered, StreamExt};
use log::{debug, info};
use rand::{seq::IteratorRandom, thread_rng};
use tokio::sync::Mutex;

use crate::{args::Args, client::TestWebSocketTransport};

#[async_trait]
pub trait Context {
    async fn init(args: &Args) -> Self;
}

#[async_trait]
pub trait Client<C: Context> {
    async fn from_rpc_client(client: RpcClient<TestWebSocketTransport>) -> Self;
    async fn act(self, context: &C) -> Self;
}

pub struct Simulation;

impl Simulation {
    pub async fn run<SC, C>(args: &Args, rpc_clients: Vec<RpcClient<TestWebSocketTransport>>)
    where
        SC: Context + Send + Sync + 'static,
        C: Client<SC> + Send + Sync + 'static,
    {
        let context = SC::init(args).await;
        let mut clients = vec![];
        for rpc_client in rpc_clients {
            clients.push(C::from_rpc_client(rpc_client).await);
        }

        let clients = Arc::new(Mutex::new(clients));
        let context = Arc::new(context);

        debug!("Simulation > Wait for 1s before start...");
        sleep(Duration::from_secs(1));
        let mut futures = FuturesUnordered::new();
        for worker_id in 0..args.parallel {
            futures.push(tokio::spawn(worker(
                worker_id,
                args.clone(),
                clients.clone(),
                context.clone(),
            )));
        }

        let mut worker_ids = (0..args.parallel).collect::<HashSet<_>>();

        while let Some(worker_result) = futures.next().await {
            match worker_result {
                Ok(worker_id) => {
                    worker_ids.remove(&worker_id);
                    debug!("Remaining active workers {}", worker_ids.len());
                }
                _ => {
                    debug!("Worker failed to join");
                }
            }
        }
    }
}

async fn worker<SC, C>(
    worker_id: u8,
    args: Args,
    clients: Arc<Mutex<Vec<C>>>,
    context: Arc<SC>,
) -> u8
where
    SC: Context + Send + Sync,
    C: Client<SC> + Send + Sync,
{
    let duration = Duration::from_secs(60 * args.duration as u64);
    let start = Instant::now();
    loop {
        debug!("Worker {worker_id} > Locking clients");
        let mut clients_guard = clients.lock().await;
        let i = (0..clients_guard.len()).choose(&mut thread_rng());
        if let Some(i) = i {
            let client = clients_guard.remove(i);

            if start.elapsed() > duration {
                debug!("Worker {worker_id} > No more time to act, dropping client!");
                info!("Worker {worker_id} > Clients left: {}", clients_guard.len());
                continue;
            }
            drop(clients_guard);
            debug!("Worker {worker_id} > Clients guard manually dropped");
            let client = client.act(&context).await;
            debug!("Worker {worker_id} > client {i} acted");
            clients.lock().await.push(client);
        } else {
            debug!("Worker {worker_id} > No more clients!");
            break;
        }

        let millis = 100;
        debug!("Worker {worker_id} > Waiting {millis} ms before next iteration");
        sleep(Duration::from_millis(millis));
    }
    info!("Worker {worker_id} > Returning");
    worker_id
}
