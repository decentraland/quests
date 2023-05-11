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
    Ok(Address::default())
    // let authenticator = Authenticator::new();
    // let (mut ws_write, mut ws_read) = ws.split();

    // let message_to_be_firmed = format!("signature_challenge_{}", fastrand::u32(..));

    // ws_write
    //     .send(Message::text(&message_to_be_firmed))
    //     .await
    //     .map_err(|_| AuthenticationError::FailedToSendChallenge)?;

    // match tokio::time::timeout(Duration::from_secs(30), ws_read.next()).await {
    //     Ok(client_response) => {
    //         if let Some(Ok(response)) = client_response {
    //             if let Ok(auth_chain) = response.to_str() {
    //                 if let Ok(auth_chain) = AuthChain::from_json(auth_chain) {
    //                     if let Ok(address) = authenticator
    //                         .verify_signature(&auth_chain, &message_to_be_firmed)
    //                         .await
    //                     {
    //                         Ok(address.to_owned())
    //                     } else {
    //                         Err(AuthenticationError::WrongSignature)
    //                     }
    //                 } else {
    //                     Err(AuthenticationError::InvalidMessage)
    //                 }
    //             } else {
    //                 Err(AuthenticationError::InvalidMessage)
    //             }
    //         } else {
    //             Err(AuthenticationError::ConnectionError)
    //         }
    //     }
    //     Err(_) => Err(AuthenticationError::Timeout),
    // }
}
