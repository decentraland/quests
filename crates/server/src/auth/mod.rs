use std::time::Duration;

use dcl_crypto::{Address, AuthChain, Authenticator};
use futures_util::{SinkExt, StreamExt};
use warp::ws::{Message, WebSocket};

#[derive(Debug)]
pub enum AuthenticationError {
    FailedToSendChallenge,
    WrongSignature,
    Timeout,
    NotTextMessage,
    ConnectionError,
}

pub async fn authenticate_dcl_user(
    ws: WebSocket,
) -> Result<(WebSocket, Address), (WebSocket, AuthenticationError)> {
    let authenticator = Authenticator::new();
    let (mut ws_write, mut ws_read) = ws.split();

    let message_to_be_firmed = format!("signature_challenge_{}", fastrand::u32(..));

    if ws_write
        .send(Message::text(&message_to_be_firmed))
        .await
        .is_err()
    {
        return Err((
            // Safe to unwrap
            ws_write.reunite(ws_read).unwrap(),
            AuthenticationError::FailedToSendChallenge,
        ));
    }

    match tokio::time::timeout(Duration::from_secs(30), ws_read.next()).await {
        Ok(client_response) => {
            if let Some(Ok(response)) = client_response {
                if let Ok(auth_chain) = response.to_str() {
                    let auth_chain = AuthChain::from_json(auth_chain).unwrap();
                    if let Ok(address) = authenticator
                        .verify_signature(&auth_chain, &message_to_be_firmed)
                        .await
                    {
                        let address = address.to_owned();

                        // Safe to unwrap
                        Ok((ws_write.reunite(ws_read).unwrap(), address))
                    } else {
                        Err((
                            // Safe to unwrap
                            ws_write.reunite(ws_read).unwrap(),
                            AuthenticationError::WrongSignature,
                        ))
                    }
                } else {
                    Err((
                        // Safe to unwrap
                        ws_write.reunite(ws_read).unwrap(),
                        AuthenticationError::NotTextMessage,
                    ))
                }
            } else {
                Err((
                    // Safe to unwrap
                    ws_write.reunite(ws_read).unwrap(),
                    AuthenticationError::ConnectionError,
                ))
            }
        }
        Err(_) => Err((
            // Safe to unwrap
            ws_write.reunite(ws_read).unwrap(),
            AuthenticationError::Timeout,
        )),
    }
}
