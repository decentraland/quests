use std::{thread::sleep, time::Duration};

use clap::Parser;
use dcl_rpc::{
    client::RpcClient,
    transports::web_socket::{WebSocketClient, WebSocketTransport},
};
use env_logger::init as initialize_logger;
use log::info;
use quests_benchmark::{
    simulation::{TestClient, TestContext},
    Args, Simulation,
};
use tokio::time::Instant;

#[tokio::main]
async fn main() {
    initialize_logger();

    let args = Args::parse();

    let test_elapsed_time = Instant::now();
    let mut set = tokio::task::JoinSet::new();

    let mut whole_conns = vec![];
    let mut client_conns = vec![];
    let mut rpc_clients = vec![];

    for i in 0..args.clients {
        set.spawn(handle_client(args.rpc_host.clone()));
        if (i + 1) % args.parallel as usize == 0 {
            while let Some(res) = set.join_next().await {
                match res.unwrap() {
                    Ok((client, whole_conn, client_conn)) => {
                        rpc_clients.push(client);
                        whole_conns.push(whole_conn);
                        client_conns.push(client_conn);

                        info!("Connected clients: {}", rpc_clients.len());
                    }
                    Err(_) => {
                        info!("Ending test as clients can't connect to server anymore");
                        return;
                    }
                }
            }
            sleep(Duration::from_millis(500));
        }
    }

    let test_elapsed_time = test_elapsed_time.elapsed().as_secs();
    let mean_whole = mean(&whole_conns);
    let mean_client_conns = mean(&client_conns);

    info!("Clients Creation >");
    info!("\nCurrent test duration: {} secs", test_elapsed_time);
    info!("\nEntire Connection (mean) {mean_whole} ms");
    info!("\nClient Connection (mean) {mean_client_conns} ms");

    info!(
        "\nSimulation > Started and will run for {} minutes...",
        args.duration
    );
    let duration = Duration::from_secs(60 * args.duration as u64);
    Simulation::run::<TestContext, TestClient>(&args, rpc_clients, duration).await;
    info!("\nSimulation > Completed");
}

pub fn mean(values: &[u128]) -> u128 {
    values.iter().sum::<u128>() / values.len() as u128
}

pub async fn handle_client(
    host: String,
) -> Result<(RpcClient<WebSocketTransport>, u128, u128), ()> {
    let whole_connection = Instant::now();
    let ws = WebSocketClient::connect(&host).await.map_err(|e| {
        log::error!("Couldn't connect to ws: {e:?}");
    })?;
    let transport = WebSocketTransport::new(ws);

    let client_connection = Instant::now();
    let client = RpcClient::new(transport).await.unwrap();
    let client_creation_elapsed = client_connection.elapsed().as_millis();
    let whole_connection = whole_connection.elapsed().as_millis();

    Ok((client, whole_connection, client_creation_elapsed))
}
