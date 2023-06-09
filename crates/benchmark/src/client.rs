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
    identity: Option<Identity>,
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
        let identity = if let Some(identity) = identity {
            identity
        } else {
            let signer = Account::random();
            Identity::from_signer(
                &signer,
                Expiration::try_from("3021-10-16T22:32:29.626Z").unwrap(),
            )
        };

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

pub fn create_test_identity() -> dcl_crypto::Identity {
    dcl_crypto::Identity::from_json(
      r#"{
     "ephemeralIdentity": {
       "address": "0x84452bbFA4ca14B7828e2F3BBd106A2bD495CD34",
       "publicKey": "0x0420c548d960b06dac035d1daf826472eded46b8b9d123294f1199c56fa235c89f2515158b1e3be0874bfb15b42d1551db8c276787a654d0b8d7b4d4356e70fe42",
       "privateKey": "0xbc453a92d9baeb3d10294cbc1d48ef6738f718fd31b4eb8085efe7b311299399"
     },
     "expiration": "3021-10-16T22:32:29.626Z",
     "authChain": [
       {
         "type": "SIGNER",
         "payload": "0x7949f9f239d1a0816ce5eb364a1f588ae9cc1bf5",
         "signature": ""
       },
       {
         "type": "ECDSA_EPHEMERAL",
         "payload": "Decentraland Login\nEphemeral address: 0x84452bbFA4ca14B7828e2F3BBd106A2bD495CD34\nExpiration: 3021-10-16T22:32:29.626Z",
         "signature": "0x39dd4ddf131ad2435d56c81c994c4417daef5cf5998258027ef8a1401470876a1365a6b79810dc0c4a2e9352befb63a9e4701d67b38007d83ffc4cd2b7a38ad51b"
       }
     ]
    }"#,
  ).unwrap()
}

pub fn get_signed_headers(
    identity: Identity,
    method: &str,
    path: &str,
    metadata: &str,
) -> Vec<(String, String)> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let payload = [method, path, &ts.to_string(), metadata]
        .join(":")
        .to_lowercase();

    let authchain = identity.sign_payload(payload);

    vec![
        (
            "X-Identity-Auth-Chain-0".to_string(),
            serde_json::to_string(authchain.get(0).unwrap()).unwrap(),
        ),
        (
            "X-Identity-Auth-Chain-1".to_string(),
            serde_json::to_string(authchain.get(1).unwrap()).unwrap(),
        ),
        (
            "X-Identity-Auth-Chain-2".to_string(),
            serde_json::to_string(authchain.get(2).unwrap()).unwrap(),
        ),
        ("X-Identity-Timestamp".to_string(), ts.to_string()),
        ("X-Identity-Metadata".to_string(), metadata.to_string()),
    ]
}
