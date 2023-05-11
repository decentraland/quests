use futures_util::StreamExt;
use log::{debug, info};
use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use clap::{command, Parser};
use dcl_rpc::{client::RpcClient, transports::web_socket::WebSocketTransport};
use futures_util::{lock::Mutex, stream::FuturesUnordered};
use rand::{seq::IteratorRandom, thread_rng};

include!(concat!(env!("OUT_DIR"), "/decentraland.quests.rs"));

pub mod quests;
pub mod simulation;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, default_value = "ws://127.0.0.1:3001", help = "Hostname")]
    pub rpc_host: String,

    #[arg(
        short,
        long,
        default_value = "http://127.0.0.1:3000",
        help = "Hostname"
    )]
    pub api_host: String,

    #[arg(short, long, default_value_t = 50, help = "Parallel")]
    pub parallel: u8,

    #[arg(
        short,
        long,
        default_value_t = 10000,
        help = "Amount of clients to connect"
    )]
    pub clients: usize,

    #[arg(
        short,
        long,
        default_value_t = 5,
        help = "Simulation duration in minutes"
    )]
    pub duration: u8,
}

#[async_trait]
pub trait Context {
    async fn init(args: &Args) -> Self;
}

#[async_trait]
pub trait Client<C: Context> {
    async fn from_rpc_client(client: RpcClient<WebSocketTransport>) -> Self;
    async fn act(self, context: &C) -> Self;
}

pub struct Simulation;

impl Simulation {
    pub async fn run<SC, C>(
        args: &Args,
        rpc_clients: Vec<RpcClient<WebSocketTransport>>,
        duration: Duration,
    ) where
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

        debug!("Simulation > Wait for 10s before start...");
        sleep(Duration::from_secs(10));
        let mut futures = FuturesUnordered::new();
        for worker_id in 0..args.parallel {
            futures.push(tokio::spawn(worker(
                worker_id as usize,
                duration,
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
