use dcl_rpc::{
    client::RpcClient,
    transports::web_socket::{WebSocketClient, WebSocketTransport},
};
use log::info;
use quests_benchmark::{
    simulation::{TestClient, TestContext},
    Simulation,
};
use tokio::time::Instant;

use env_logger::init as initialize_logger;

fn mean(numbers: &Vec<u128>) -> u128 {
    let sum: u128 = numbers.iter().sum();

    sum / numbers.len() as u128
}

#[tokio::main]
async fn main() {
    initialize_logger();

    let test_elapsed_time = Instant::now();
    let mut set = tokio::task::JoinSet::new();

    let max_clients = 28000;
    let concurrency = 100;
    let mut whole_conns = vec![];
    let mut client_conns = vec![];
    let mut rpc_clients = vec![];

    for i in 0..max_clients {
        set.spawn(handle_client());
        if (i + 1) % concurrency == 0 {
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
        }
    }

    let test_elapsed_time = test_elapsed_time.elapsed().as_secs();
    let mean_whole = mean(&whole_conns);
    let mean_client_conns = mean(&client_conns);

    info!("Clients Creation >");
    info!("\nCurrent test duration: {} secs", test_elapsed_time);
    info!("\nEntire Connection (mean) {mean_whole} ms");
    info!("\nClient Connection (mean) {mean_client_conns} ms");

    info!("\nSimulation > Started");
    Simulation::run::<TestContext, TestClient>(rpc_clients).await;
    info!("\nSimulation > Completed");
}

pub async fn handle_client() -> Result<(RpcClient<WebSocketTransport>, u128, u128), ()> {
    let whole_connection = Instant::now();
    let ws = WebSocketClient::connect("ws://0.0.0.0:3001")
        .await
        .map_err(|e| {
            println!("Couldn't connect to ws: {e:?}");
        })?;
    let transport = WebSocketTransport::new(ws);

    let client_connection = Instant::now();
    let client = RpcClient::new(transport).await.unwrap();
    let client_creation_elapsed = client_connection.elapsed().as_millis();
    let whole_connection = whole_connection.elapsed().as_millis();

    Ok((client, whole_connection, client_creation_elapsed))
}
