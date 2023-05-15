use dcl_crypto::Address;
use dcl_rpc::transports::{Transport, TransportError, TransportMessage};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use log::{debug, error};
use tokio::sync::Mutex;
use tungstenite::Error as WsError;
use warp::ws::{Message as WarpWSMessage, WebSocket};

type ReadStream = SplitStream<WebSocket>;
type WriteStream = SplitSink<WebSocket, WarpWSMessage>;

pub struct WarpWebSocketTransport {
    read: Mutex<ReadStream>,
    write: Mutex<WriteStream>,
    pub user_address: Address,
}

impl WarpWebSocketTransport {
    /// Crates a new [`WebSocketTransport`] from a websocket connection generated by [`WebSocketServer`] or [`WebSocketClient`]
    pub fn new(ws: WebSocket, address: Address) -> Self {
        let (write, read) = ws.split();
        Self {
            read: Mutex::new(read),
            write: Mutex::new(write),
            user_address: address,
        }
    }
}

#[async_trait::async_trait]
impl Transport for WarpWebSocketTransport {
    async fn receive(&self) -> Result<TransportMessage, TransportError> {
        loop {
            match self.read.lock().await.next().await {
                Some(Ok(message)) => {
                    if message.is_binary() {
                        let message_data = message.into_bytes();
                        return Ok(message_data);
                    } else if message.is_ping() || message.is_pong() {
                        continue;
                    } else {
                        if message.is_close() {
                            return Err(TransportError::Closed);
                        }
                        // Ignore messages that are not binary
                        debug!("> WebSocketTransport > Received message is not binary");
                        return Err(TransportError::NotBinaryMessage);
                    }
                }
                Some(Err(err)) => {
                    debug!(
                        "> WebSocketTransport > Failed to receive message {}",
                        err.to_string()
                    );
                    return Err(return_ws_error(err));
                }
                None => {
                    debug!("> WebSocketTransport > None received > Closing...");
                    return Err(TransportError::Closed);
                }
            }
        }
    }

    async fn send(&self, message: Vec<u8>) -> Result<(), TransportError> {
        let message = WarpWSMessage::binary(message);
        match self.write.lock().await.send(message).await {
            Err(err) => {
                debug!(
                    "> WebSocketTransport > Error on sending in a ws connection {}",
                    err.to_string()
                );
                Err(return_ws_error(err))
            }
            Ok(_) => Ok(()),
        }
    }

    async fn close(&self) {
        match self.write.lock().await.close().await {
            Ok(_) => {
                debug!("> WebSocketTransport > Closed successfully")
            }
            Err(err) => {
                error!("> WebSocketTransport > Error: Couldn't close tranport: {err:?}")
            }
        }
    }
}

fn translate_warp_error(err: &warp::Error) -> Option<TransportError> {
    use std::error::Error;
    if let Some(source_error) = err.source() {
        if let Some(error) = source_error.downcast_ref::<WsError>() {
            match error {
                WsError::ConnectionClosed | WsError::AlreadyClosed => Some(TransportError::Closed),
                _ => None,
            };
        }
    }
    None
}

fn return_ws_error(err: warp::Error) -> TransportError {
    translate_warp_error(&err).unwrap_or(TransportError::Internal(Box::new(err)))
}
