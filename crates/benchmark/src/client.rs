use std::time::{Duration, Instant};

use dcl_crypto::{
    account::{Account, Signer},
    AuthChain,
};
use dcl_rpc::{
    client::RpcClient,
    transports::web_socket::{WebSocketClient, WebSocketTransport},
};
use tungstenite::Message;

use crate::args::Args;

#[derive(Debug)]
pub enum ClientCreationError {
    Authentication,
    Connection,
    Transport,
}

pub async fn handle_client(
    args: Args,
) -> Result<(RpcClient<WebSocketTransport>, u128, u128), ClientCreationError> {
    let Args {
        rpc_host,
        authenticate,
        ..
    } = args;
    let whole_connection = Instant::now();
    let ws = WebSocketClient::connect(&rpc_host).await.map_err(|e| {
        log::error!("Couldn't connect to ws: {e:?}");
        ClientCreationError::Connection
    })?;

    let ws = if authenticate {
        match ws.receive().await {
            Some(Ok(Message::Text(challenge))) => {
                let account = Account::random();
                let signature = account.sign(&challenge);
                let auth_chain =
                    AuthChain::simple(account.address(), &challenge, signature.to_string())
                        .map_err(|_| ClientCreationError::Authentication)?;
                let auth_chain = serde_json::to_string(&auth_chain)
                    .map_err(|_| ClientCreationError::Authentication)?;
                ws.send(Message::Text(auth_chain))
                    .await
                    .map_err(|_| ClientCreationError::Authentication)?;
                ws.clone().ping_every(Duration::from_secs(30)).await;
                ws
            }
            _ => return Err(ClientCreationError::Authentication),
        }
    } else {
        ws.clone().ping_every(Duration::from_secs(30)).await;
        ws
    };

    let transport = WebSocketTransport::new(ws);

    let client_connection = Instant::now();
    let client = RpcClient::new(transport)
        .await
        .map_err(|_| ClientCreationError::Transport)?;
    let client_creation_elapsed = client_connection.elapsed().as_millis();
    let whole_connection = whole_connection.elapsed().as_millis();

    Ok((client, whole_connection, client_creation_elapsed))
}
