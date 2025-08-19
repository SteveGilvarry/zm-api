use crate::error::{AppError, AppResult};
use futures::{SinkExt, StreamExt};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::connect_async;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::{Error as WsError, Message};

/// WebSocket client wrapping sender and receiver halves
pub struct WsClient {
    sender: futures::stream::SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    receiver: futures::stream::SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl WsClient {
    /// Connect to the given `url`, return a new `WsClient`
    pub async fn connect(url: impl AsRef<str>) -> AppResult<Self> {
        let (ws_stream, _) = connect_async(url.as_ref())
            .await
            .map_err(AppError::WebSocketError)?;
        let (sender, receiver) = ws_stream.split();
        Ok(Self { sender, receiver })
    }

    /// Send a JSON-serializable message as a Text frame
    pub async fn send_json<T>(&mut self, msg: &T) -> AppResult<()>
    where
        T: Serialize,
    {
        let txt = serde_json::to_string(msg)
            .map_err(AppError::ParseJsonError)?;
        self.sender
            .send(txt.into())
            .await
            .map_err(AppError::WebSocketError)?;
        Ok(())
    }

    /// Receive and deserialize the next JSON message from the WebSocket
    pub async fn recv_json<T>(&mut self) -> AppResult<T>
    where
        T: DeserializeOwned,
    {
        while let Some(frame) = self.receiver.next().await {
            let msg = frame.map_err(AppError::WebSocketError)?;
            match msg {
                Message::Text(txt) => {
                    return serde_json::from_str(&txt)
                        .map_err(AppError::ParseJsonError);
                }
                Message::Binary(bin) => {
                    return serde_json::from_slice(&bin)
                        .map_err(AppError::ParseJsonError);
                }
                Message::Ping(p) => {
                    let _ = self.sender.send(Message::Pong(p)).await;
                }
                Message::Pong(_) => continue,
                Message::Close(_) => return Err(AppError::WebSocketError(WsError::ConnectionClosed)),
                _ => continue,
            }
        }
        Err(AppError::WebSocketError(WsError::ConnectionClosed))
    }
}
