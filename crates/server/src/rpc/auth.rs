use std::time::Duration;

use dcl_crypto::{Address, AuthChain, Authenticator};
use futures_util::{SinkExt, StreamExt};
use warp::ws::{Message, WebSocket};

#[derive(Debug)]
pub enum AuthenticationError {
    FailedToSendChallenge,
    WrongSignature,
    Timeout,
    InvalidMessage,
    ConnectionError,
}

pub async fn authenticate_dcl_user(ws: &mut WebSocket) -> Result<Address, AuthenticationError> {
    let authenticator = Authenticator::new();
    let (mut ws_write, mut ws_read) = ws.split();

    let message_to_be_firmed = format!("signature_challenge_{}", fastrand::u32(..));

    ws_write
        .send(Message::text(&message_to_be_firmed))
        .await
        .map_err(|_| AuthenticationError::FailedToSendChallenge)?;

    match tokio::time::timeout(Duration::from_secs(30), ws_read.next()).await {
        Ok(Some(Ok(client_response))) => {
            let message = client_response
                .to_str()
                .map_err(|_| AuthenticationError::InvalidMessage)?;
            let auth_chain = AuthChain::from_json(message).map_err(|e| {
                log::debug!("Invalid auth_chain: {message}: {e:?}");
                AuthenticationError::InvalidMessage
            })?;
            authenticator
                .verify_signature(&auth_chain, &message_to_be_firmed)
                .await
                .map(|address| address.to_owned())
                .map_err(|e| {
                    log::debug!("Invalid signature: {e:?}");
                    AuthenticationError::WrongSignature
                })
        }
        Ok(_) => Err(AuthenticationError::ConnectionError),
        Err(_) => Err(AuthenticationError::Timeout),
    }
}
