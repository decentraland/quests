use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use dcl_crypto::{account::Account, Expiration, Identity};
use dcl_rpc::{
    client::RpcClient,
    transports::web_sockets::{
        tungstenite::{TungsteniteWebSocket, WebSocketClient},
        Message, WebSocket, WebSocketTransport,
    },
};

use crate::args::Args;

pub type TestWebSocketTransport = WebSocketTransport<TungsteniteWebSocket, ()>;

#[derive(Debug)]
pub enum ClientCreationError {
    Authentication,
    Connection,
    Transport,
}

pub async fn handle_client(
    args: Args,
) -> Result<(RpcClient<TestWebSocketTransport>, u128, u128), ClientCreationError> {
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
        let signer = Account::random();
        let identity = Identity::from_signer(
            &signer,
            Expiration::try_from("3021-10-16T22:32:29.626Z").unwrap(),
        );
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let path = "/";
        let method = "get";
        let metadata = "{}";
        let auth_chain = identity.sign_payload(format!("{method}:{path}:{now}:{metadata}"));
        let signed_headers = format!(
            r#"{{"X-Identity-Auth-Chain-0": {},  "X-Identity-Auth-Chain-1": {},  "X-Identity-Auth-Chain-2": {}, "X-Identity-Timestamp": {}, "X-Identity-Metadata": {} }}"#,
            serde_json::to_string(auth_chain.get(0).unwrap()).unwrap(),
            serde_json::to_string(auth_chain.get(1).unwrap()).unwrap(),
            serde_json::to_string(auth_chain.get(2).unwrap()).unwrap(),
            now,
            "{}"
        );

        ws.send(Message::Text(signed_headers))
            .await
            .map_err(|_| ClientCreationError::Authentication)?;

        ws
    } else {
        ws
    };

    ws.clone().ping_every(Duration::from_secs(30)).await;
    let transport = WebSocketTransport::new(ws);

    let client_connection = Instant::now();
    let client = RpcClient::new(transport)
        .await
        .map_err(|_| ClientCreationError::Transport)?;
    let client_creation_elapsed = client_connection.elapsed().as_millis();
    let whole_connection = whole_connection.elapsed().as_millis();

    Ok((client, whole_connection, client_creation_elapsed))
}
