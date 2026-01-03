use axum::{
    extract::{ws::WebSocket, Path, State, WebSocketUpgrade},
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message as TungsteniteMessage};
use tracing::{debug, error, info, warn};

use crate::{
    error::{AppError, AppResult},
    server::state::AppState,
};

/// Proxy WebSocket connection to go2rtc for a specific monitor
///
/// Client connects to: wss://zm-api/api/v3/go2rtc/{monitor_id}/ws
/// zm-api proxies to:  ws://localhost:1984/api/ws?src=zm{monitor_id}
///
/// Authentication: JWT token required (handled by middleware)
/// Authorization: User must have permission to view this monitor (future enhancement)
#[utoipa::path(
    get,
    path = "/api/v3/go2rtc/{monitor_id}/ws",
    params(
        ("monitor_id" = u32, Path, description = "Monitor ID")
    ),
    responses(
        (status = 101, description = "WebSocket connection established"),
        (status = 401, description = "Unauthorized", body = crate::error::AppResponseError),
        (status = 403, description = "Forbidden - no access to this monitor", body = crate::error::AppResponseError),
        (status = 502, description = "go2rtc backend unavailable", body = crate::error::AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "Streaming"
)]
pub async fn go2rtc_ws_proxy(
    State(state): State<AppState>,
    Path(monitor_id): Path<u32>,
    ws: WebSocketUpgrade,
) -> Response {
    info!("WebSocket proxy request for monitor {}", monitor_id);

    // Future enhancement: Check user permissions for this monitor
    // For now, we rely on JWT authentication from middleware

    // Upgrade the client connection and handle proxying
    ws.on_upgrade(move |socket| handle_ws_proxy(socket, monitor_id, state))
}

/// Proxy for specific go2rtc stream types (webrtc, mse, mp4)
///
/// This endpoint allows specifying the stream type explicitly for go2rtc's
/// WebSocket API.
#[utoipa::path(
    get,
    path = "/api/v3/go2rtc/{monitor_id}/ws/{stream_type}",
    params(
        ("monitor_id" = u32, Path, description = "Monitor ID"),
        ("stream_type" = String, Path, description = "Stream type: webrtc, mse, or mp4")
    ),
    responses(
        (status = 101, description = "WebSocket connection established"),
        (status = 400, description = "Invalid stream type", body = crate::error::AppResponseError),
        (status = 401, description = "Unauthorized", body = crate::error::AppResponseError),
        (status = 502, description = "go2rtc backend unavailable", body = crate::error::AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "Streaming"
)]
pub async fn go2rtc_typed_ws_proxy(
    State(state): State<AppState>,
    Path((monitor_id, stream_type)): Path<(u32, String)>,
    ws: WebSocketUpgrade,
) -> AppResult<Response> {
    // Validate stream type
    match stream_type.as_str() {
        "webrtc" | "mse" | "mp4" => {
            info!(
                "WebSocket proxy request for monitor {} with stream type {}",
                monitor_id, stream_type
            );

            // Upgrade the client connection and handle proxying
            Ok(ws.on_upgrade(move |socket| {
                handle_typed_ws_proxy(socket, monitor_id, stream_type, state)
            }))
        }
        _ => {
            warn!("Invalid stream type requested: {}", stream_type);
            Err(AppError::BadRequestError(format!(
                "Invalid stream type '{}'. Must be one of: webrtc, mse, mp4",
                stream_type
            )))
        }
    }
}

/// Handle the bidirectional WebSocket proxying
async fn handle_ws_proxy(client_socket: WebSocket, monitor_id: u32, state: AppState) {
    // Build go2rtc WebSocket URL - ALWAYS localhost for security
    let go2rtc_url = build_go2rtc_url(monitor_id, None, &state);

    info!(
        "Establishing proxy connection to go2rtc for monitor {}: {}",
        monitor_id, go2rtc_url
    );

    // Connect to go2rtc backend
    let backend_result = connect_async(&go2rtc_url).await;

    let (backend_socket, _) = match backend_result {
        Ok(result) => result,
        Err(e) => {
            error!(
                "Failed to connect to go2rtc backend for monitor {}: {}",
                monitor_id, e
            );
            // Try to inform the client before closing
            let mut client_socket = client_socket;
            let _ = client_socket
                .send(axum::extract::ws::Message::Close(Some(
                    axum::extract::ws::CloseFrame {
                        code: 1011, // Internal error
                        reason: std::borrow::Cow::from("Backend unavailable"),
                    },
                )))
                .await;
            return;
        }
    };

    info!(
        "Successfully connected to go2rtc backend for monitor {}",
        monitor_id
    );

    // Start bidirectional proxying
    proxy_websocket_bidirectional(client_socket, backend_socket, monitor_id).await;
}

/// Handle typed WebSocket proxying with stream type parameter
async fn handle_typed_ws_proxy(
    client_socket: WebSocket,
    monitor_id: u32,
    stream_type: String,
    state: AppState,
) {
    // Build go2rtc WebSocket URL with stream type - ALWAYS localhost for security
    let go2rtc_url = build_go2rtc_url(monitor_id, Some(&stream_type), &state);

    info!(
        "Establishing typed proxy connection to go2rtc for monitor {} ({}): {}",
        monitor_id, stream_type, go2rtc_url
    );

    // Connect to go2rtc backend
    let backend_result = connect_async(&go2rtc_url).await;

    let (backend_socket, _) = match backend_result {
        Ok(result) => result,
        Err(e) => {
            error!(
                "Failed to connect to go2rtc backend for monitor {} ({}): {}",
                monitor_id, stream_type, e
            );
            // Try to inform the client before closing
            let mut client_socket = client_socket;
            let _ = client_socket
                .send(axum::extract::ws::Message::Close(Some(
                    axum::extract::ws::CloseFrame {
                        code: 1011, // Internal error
                        reason: std::borrow::Cow::from("Backend unavailable"),
                    },
                )))
                .await;
            return;
        }
    };

    info!(
        "Successfully connected to go2rtc backend for monitor {} ({})",
        monitor_id, stream_type
    );

    // Start bidirectional proxying
    proxy_websocket_bidirectional(client_socket, backend_socket, monitor_id).await;
}

/// Build the go2rtc WebSocket URL
///
/// SECURITY: This function ALWAYS uses localhost:1984 to prevent URL injection attacks.
/// The monitor_id is validated as u32, and stream_type is validated against a whitelist.
fn build_go2rtc_url(monitor_id: u32, stream_type: Option<&str>, state: &AppState) -> String {
    // SECURITY: Always use localhost - never accept user-provided URLs
    // go2rtc should be configured to bind ONLY to localhost:1984
    let base_url = "ws://127.0.0.1:1984";

    // Build the source parameter - using ZoneMinder naming convention
    let src = format!("zm{}", monitor_id);

    // Build URL with optional stream type
    match stream_type {
        Some(stype) => format!("{}/api/ws?src={}&type={}", base_url, src, stype),
        None => format!("{}/api/ws?src={}", base_url, src),
    }
}

/// Perform bidirectional WebSocket proxying
///
/// This function splits both WebSocket connections and creates two tasks:
/// 1. Forward messages from client to go2rtc backend
/// 2. Forward messages from go2rtc backend to client
///
/// When either direction closes or errors, both connections are cleaned up.
async fn proxy_websocket_bidirectional(
    client_socket: WebSocket,
    backend_socket: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    monitor_id: u32,
) {
    // Split both sockets into sender and receiver halves
    let (mut client_tx, mut client_rx) = client_socket.split();
    let (mut backend_tx, mut backend_rx) = backend_socket.split();

    // Task 1: Forward client -> backend
    let client_to_backend = async move {
        while let Some(msg) = client_rx.next().await {
            match msg {
                Ok(client_msg) => {
                    // Convert Axum WebSocket message to tungstenite message
                    let backend_msg = match client_msg {
                        axum::extract::ws::Message::Text(text) => {
                            debug!(
                                "Client->Backend text message for monitor {}: {} bytes",
                                monitor_id,
                                text.len()
                            );
                            TungsteniteMessage::Text(text)
                        }
                        axum::extract::ws::Message::Binary(data) => {
                            debug!(
                                "Client->Backend binary message for monitor {}: {} bytes",
                                monitor_id,
                                data.len()
                            );
                            TungsteniteMessage::Binary(data)
                        }
                        axum::extract::ws::Message::Ping(data) => {
                            debug!("Client->Backend ping for monitor {}", monitor_id);
                            TungsteniteMessage::Ping(data)
                        }
                        axum::extract::ws::Message::Pong(data) => {
                            debug!("Client->Backend pong for monitor {}", monitor_id);
                            TungsteniteMessage::Pong(data)
                        }
                        axum::extract::ws::Message::Close(frame) => {
                            info!("Client closed connection for monitor {}", monitor_id);
                            if let Some(cf) = frame {
                                TungsteniteMessage::Close(Some(tokio_tungstenite::tungstenite::protocol::CloseFrame {
                                    code: tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::from(cf.code),
                                    reason: cf.reason,
                                }))
                            } else {
                                TungsteniteMessage::Close(None)
                            }
                        }
                    };

                    // Forward to backend
                    if let Err(e) = backend_tx.send(backend_msg).await {
                        error!(
                            "Failed to forward message to go2rtc backend for monitor {}: {}",
                            monitor_id, e
                        );
                        break;
                    }
                }
                Err(e) => {
                    error!(
                        "Error receiving from client for monitor {}: {}",
                        monitor_id, e
                    );
                    break;
                }
            }
        }
        info!(
            "Client->Backend forwarding finished for monitor {}",
            monitor_id
        );
    };

    // Task 2: Forward backend -> client
    let backend_to_client = async move {
        while let Some(msg) = backend_rx.next().await {
            match msg {
                Ok(backend_msg) => {
                    // Convert tungstenite message to Axum WebSocket message
                    let client_msg = match backend_msg {
                        TungsteniteMessage::Text(text) => {
                            debug!(
                                "Backend->Client text message for monitor {}: {} bytes",
                                monitor_id,
                                text.len()
                            );
                            axum::extract::ws::Message::Text(text)
                        }
                        TungsteniteMessage::Binary(data) => {
                            debug!(
                                "Backend->Client binary message for monitor {}: {} bytes",
                                monitor_id,
                                data.len()
                            );
                            axum::extract::ws::Message::Binary(data)
                        }
                        TungsteniteMessage::Ping(data) => {
                            debug!("Backend->Client ping for monitor {}", monitor_id);
                            axum::extract::ws::Message::Ping(data)
                        }
                        TungsteniteMessage::Pong(data) => {
                            debug!("Backend->Client pong for monitor {}", monitor_id);
                            axum::extract::ws::Message::Pong(data)
                        }
                        TungsteniteMessage::Close(frame) => {
                            info!("Backend closed connection for monitor {}", monitor_id);
                            if let Some(cf) = frame {
                                axum::extract::ws::Message::Close(Some(
                                    axum::extract::ws::CloseFrame {
                                        code: cf.code.into(),
                                        reason: cf.reason,
                                    },
                                ))
                            } else {
                                axum::extract::ws::Message::Close(None)
                            }
                        }
                        TungsteniteMessage::Frame(_) => {
                            // Raw frames are not typically used in application code
                            continue;
                        }
                    };

                    // Forward to client
                    if let Err(e) = client_tx.send(client_msg).await {
                        error!(
                            "Failed to forward message to client for monitor {}: {}",
                            monitor_id, e
                        );
                        break;
                    }
                }
                Err(e) => {
                    error!(
                        "Error receiving from go2rtc backend for monitor {}: {}",
                        monitor_id, e
                    );
                    break;
                }
            }
        }
        info!(
            "Backend->Client forwarding finished for monitor {}",
            monitor_id
        );
    };

    // Run both forwarding tasks concurrently
    // When either completes (connection closed or error), cancel the other
    tokio::select! {
        _ = client_to_backend => {
            info!("Client->Backend task completed for monitor {}", monitor_id);
        }
        _ = backend_to_client => {
            info!("Backend->Client task completed for monitor {}", monitor_id);
        }
    }

    info!("WebSocket proxy session ended for monitor {}", monitor_id);
}
