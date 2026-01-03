//! Native WebRTC handlers for Phase 2 implementation.
//!
//! These handlers use the native webrtc-rs library for WebRTC connections
//! instead of the external plugin.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    response::Response,
    Json,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, warn};
use utoipa::ToSchema;

use crate::{
    error::{AppError, AppResponseError, AppResult, Resource, ResourceType},
    server::state::AppState,
    streaming::webrtc::{
        session::SessionState,
        signaling::{error_codes, ClientMessage, ServerMessage},
        PeerConnectionParams,
    },
};

/// Query parameters for WebSocket signaling connection
#[derive(Debug, Deserialize, ToSchema)]
pub struct SignalingQuery {
    /// Optional client identifier for tracking
    pub client_id: Option<String>,
}

/// Response for session info
#[derive(Debug, Serialize, ToSchema)]
pub struct SessionInfoResponse {
    pub session_id: String,
    pub monitor_id: u32,
    pub state: String,
    pub duration_seconds: u64,
    pub bytes_sent: u64,
    pub packets_sent: u64,
}

/// Response for native WebRTC stats
#[derive(Debug, Serialize, ToSchema)]
pub struct NativeWebRtcStatsResponse {
    pub active_sessions: usize,
    pub engine_status: String,
}

/// Request body for SDP offer (REST fallback)
#[derive(Debug, Deserialize, ToSchema)]
pub struct OfferRequest {
    pub sdp: String,
}

/// Response for SDP answer
#[derive(Debug, Serialize, ToSchema)]
pub struct AnswerResponse {
    pub sdp: String,
    pub session_id: String,
}

/// Request body for ICE candidate
#[derive(Debug, Deserialize, ToSchema)]
pub struct IceCandidateRequest {
    pub candidate: String,
    pub sdp_mid: Option<String>,
    pub sdp_mline_index: Option<u16>,
}

/// Success response
#[derive(Debug, Serialize, ToSchema)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: String,
}

/// WebSocket signaling endpoint for native WebRTC
#[utoipa::path(
    get,
    path = "/api/v3/webrtc/native/{monitor_id}/signaling",
    operation_id = "nativeWebRtcSignaling",
    tag = "Native WebRTC",
    params(
        ("monitor_id" = u32, Path, description = "Monitor ID"),
        ("client_id" = Option<String>, Query, description = "Optional client identifier")
    ),
    responses(
        (status = 101, description = "WebSocket upgrade for signaling"),
        (status = 401, description = "Unauthorized", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
pub async fn signaling_websocket(
    ws: WebSocketUpgrade,
    Path(monitor_id): Path<u32>,
    Query(params): Query<SignalingQuery>,
    State(state): State<AppState>,
) -> Response {
    let client_id = params.client_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    info!(
        "Native WebRTC signaling WebSocket upgrade for monitor {} from client {}",
        monitor_id, client_id
    );

    ws.on_upgrade(move |socket| handle_signaling(socket, monitor_id, client_id, state))
}

async fn handle_signaling(socket: WebSocket, monitor_id: u32, client_id: String, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Get the native WebRTC components from state
    let engine = match state.native_webrtc_engine.as_ref() {
        Some(e) => e.clone(),
        None => {
            let error_msg = ServerMessage::error(
                None,
                error_codes::INTERNAL_ERROR,
                "Native WebRTC engine not available",
            );
            let _ = sender.send(Message::Text(error_msg.to_json().into())).await;
            return;
        }
    };

    let session_manager = match state.native_session_manager.as_ref() {
        Some(s) => s.clone(),
        None => {
            let error_msg = ServerMessage::error(
                None,
                error_codes::INTERNAL_ERROR,
                "Session manager not available",
            );
            let _ = sender.send(Message::Text(error_msg.to_json().into())).await;
            return;
        }
    };

    // Create peer connection
    let pc_result = match engine
        .create_peer_connection(PeerConnectionParams {
            monitor_id,
            enable_audio: false,
        })
        .await
    {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to create peer connection: {}", e);
            let error_msg = ServerMessage::error(
                None,
                error_codes::INTERNAL_ERROR,
                format!("Failed to create peer connection: {}", e),
            );
            let _ = sender.send(Message::Text(error_msg.to_json().into())).await;
            return;
        }
    };

    // Create session with the peer connection
    let session_id = match session_manager.create_session(
        monitor_id,
        Some(client_id.clone()),
        pc_result.peer_connection.clone(),
    ) {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to create session: {}", e);
            let error_msg = ServerMessage::error(
                None,
                error_codes::MAX_SESSIONS,
                format!("Failed to create session: {}", e),
            );
            let _ = sender.send(Message::Text(error_msg.to_json().into())).await;
            return;
        }
    };

    let peer_connection = pc_result.peer_connection;

    info!(
        "Created native WebRTC session {} for monitor {} client {}",
        session_id, monitor_id, client_id
    );

    // Send connected message
    let connected_msg = ServerMessage::Connected {
        session_id: session_id.clone(),
        monitor_id,
    };
    let _ = sender
        .send(Message::Text(connected_msg.to_json().into()))
        .await;

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                match ClientMessage::parse(&text) {
                    Ok(client_msg) => {
                        let response = process_client_message(
                            client_msg,
                            &session_id,
                            &engine,
                            &session_manager,
                            &peer_connection,
                        )
                        .await;

                        if let Some(server_msg) = response {
                            if let Err(e) = sender
                                .send(Message::Text(server_msg.to_json().into()))
                                .await
                            {
                                error!("Failed to send response: {}", e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse client message: {}", e);
                        let error_msg = ServerMessage::error(
                            Some(session_id.clone()),
                            error_codes::INVALID_MESSAGE,
                            format!("Invalid message format: {}", e),
                        );
                        let _ = sender.send(Message::Text(error_msg.to_json().into())).await;
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!(
                    "Client closed WebSocket connection for session {}",
                    session_id
                );
                break;
            }
            Ok(Message::Ping(data)) => {
                let _ = sender.send(Message::Pong(data)).await;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    // Cleanup session
    session_manager.remove_session(&session_id);
    info!("Cleaned up session {}", session_id);
}

async fn process_client_message(
    msg: ClientMessage,
    session_id: &String,
    engine: &Arc<crate::streaming::webrtc::WebRtcEngine>,
    session_manager: &Arc<crate::streaming::webrtc::session::SessionManager>,
    peer_connection: &webrtc::peer_connection::RTCPeerConnection,
) -> Option<ServerMessage> {
    match msg {
        ClientMessage::Offer { sdp, .. } => {
            info!("Processing SDP offer for session {}", session_id);

            // Update session state
            let _ = session_manager.update_state(session_id, SessionState::Connecting);

            // Process the offer and generate answer
            match engine.process_offer(peer_connection, &sdp).await {
                Ok(answer_sdp) => {
                    let _ = session_manager.update_state(session_id, SessionState::Connected);

                    Some(ServerMessage::Answer {
                        session_id: session_id.to_string(),
                        sdp: answer_sdp,
                    })
                }
                Err(e) => {
                    error!("Failed to process offer: {}", e);
                    let _ = session_manager.update_state(session_id, SessionState::Failed);

                    Some(ServerMessage::error(
                        Some(session_id.to_string()),
                        error_codes::INTERNAL_ERROR,
                        format!("Failed to process offer: {}", e),
                    ))
                }
            }
        }
        ClientMessage::Answer { sdp, .. } => {
            info!("Processing SDP answer for session {}", session_id);

            match engine.process_answer(peer_connection, &sdp).await {
                Ok(_) => {
                    let _ = session_manager.update_state(session_id, SessionState::Connected);
                    None // No response needed for answer
                }
                Err(e) => {
                    error!("Failed to process answer: {}", e);
                    Some(ServerMessage::error(
                        Some(session_id.to_string()),
                        error_codes::INTERNAL_ERROR,
                        format!("Failed to process answer: {}", e),
                    ))
                }
            }
        }
        ClientMessage::IceCandidate {
            candidate,
            sdp_mid,
            sdp_mline_index,
            ..
        } => {
            info!("Processing ICE candidate for session {}", session_id);

            match engine
                .add_ice_candidate(
                    peer_connection,
                    &candidate,
                    sdp_mid.as_deref(),
                    sdp_mline_index,
                )
                .await
            {
                Ok(_) => None, // No response needed for ICE candidates
                Err(e) => {
                    warn!("Failed to add ICE candidate: {}", e);
                    Some(ServerMessage::error(
                        Some(session_id.to_string()),
                        error_codes::ICE_FAILED,
                        format!("Failed to add ICE candidate: {}", e),
                    ))
                }
            }
        }
        ClientMessage::Hangup { .. } => {
            info!("Received hangup for session {}", session_id);
            let _ = session_manager.update_state(session_id, SessionState::Disconnected);

            Some(ServerMessage::Disconnected {
                session_id: session_id.to_string(),
                reason: "Client hangup".to_string(),
            })
        }
        ClientMessage::GetStats { .. } => {
            if let Some(stats) = session_manager.get_session_stats(session_id).await {
                Some(ServerMessage::Stats {
                    session_id: session_id.to_string(),
                    stats: serde_json::json!({
                        "state": stats.state,
                        "duration_seconds": stats.duration_seconds,
                        "bytes_sent": stats.bytes_sent,
                        "packets_sent": stats.packets_sent,
                    }),
                })
            } else {
                Some(ServerMessage::error(
                    Some(session_id.to_string()),
                    error_codes::SESSION_NOT_FOUND,
                    "Session not found",
                ))
            }
        }
    }
}

/// REST endpoint for SDP offer (fallback when WebSocket not available)
#[utoipa::path(
    post,
    path = "/api/v3/webrtc/native/{monitor_id}/offer",
    operation_id = "nativeWebRtcOffer",
    tag = "Native WebRTC",
    params(
        ("monitor_id" = u32, Path, description = "Monitor ID")
    ),
    request_body = OfferRequest,
    responses(
        (status = 200, description = "SDP answer", body = AnswerResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 401, description = "Unauthorized", body = AppResponseError),
        (status = 503, description = "Native WebRTC not available", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
pub async fn handle_offer(
    State(state): State<AppState>,
    Path(monitor_id): Path<u32>,
    Json(req): Json<OfferRequest>,
) -> AppResult<Json<AnswerResponse>> {
    let engine = state.native_webrtc_engine.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailableError("Native WebRTC engine not available".to_string())
    })?;

    let session_manager = state.native_session_manager.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailableError("Session manager not available".to_string())
    })?;

    // Create peer connection
    let pc_result = engine
        .create_peer_connection(PeerConnectionParams {
            monitor_id,
            enable_audio: false,
        })
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to create peer connection: {}", e)))?;

    // Create session with the peer connection
    let session_id = session_manager
        .create_session(monitor_id, None, pc_result.peer_connection.clone())
        .map_err(|e| AppError::InternalServerError(format!("Failed to create session: {}", e)))?;

    // Process the offer
    let answer_sdp = engine
        .process_offer(&pc_result.peer_connection, &req.sdp)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to process offer: {}", e)))?;

    let _ = session_manager.update_state(&session_id, SessionState::Connected);

    Ok(Json(AnswerResponse {
        sdp: answer_sdp,
        session_id,
    }))
}

/// Add ICE candidate (REST fallback)
#[utoipa::path(
    post,
    path = "/api/v3/webrtc/native/{monitor_id}/{session_id}/candidate",
    operation_id = "nativeWebRtcCandidate",
    tag = "Native WebRTC",
    params(
        ("monitor_id" = u32, Path, description = "Monitor ID"),
        ("session_id" = String, Path, description = "Session ID")
    ),
    request_body = IceCandidateRequest,
    responses(
        (status = 200, description = "ICE candidate added", body = SuccessResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 401, description = "Unauthorized", body = AppResponseError),
        (status = 404, description = "Session not found", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
pub async fn add_ice_candidate(
    State(state): State<AppState>,
    Path((monitor_id, session_id)): Path<(u32, String)>,
    Json(req): Json<IceCandidateRequest>,
) -> AppResult<Json<SuccessResponse>> {
    let engine = state.native_webrtc_engine.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailableError("Native WebRTC engine not available".to_string())
    })?;

    let session_manager = state.native_session_manager.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailableError("Session manager not available".to_string())
    })?;

    // Get session and peer connection
    let session_lock = session_manager.get_session(&session_id).ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("session_id".to_string(), session_id.clone())],
            resource_type: ResourceType::Message,
        })
    })?;

    let session = session_lock.read().await;
    let peer_connection = &session.peer_connection;

    engine
        .add_ice_candidate(
            peer_connection,
            &req.candidate,
            req.sdp_mid.as_deref(),
            req.sdp_mline_index,
        )
        .await
        .map_err(|e| AppError::BadRequestError(format!("Invalid ICE candidate: {}", e)))?;

    info!(
        "Added ICE candidate for monitor {} session {}",
        monitor_id, session_id
    );

    Ok(Json(SuccessResponse {
        success: true,
        message: "ICE candidate added".to_string(),
    }))
}

/// Close a WebRTC session
#[utoipa::path(
    delete,
    path = "/api/v3/webrtc/native/{session_id}",
    operation_id = "closeNativeWebRtcSession",
    tag = "Native WebRTC",
    params(
        ("session_id" = String, Path, description = "Session ID")
    ),
    responses(
        (status = 200, description = "Session closed", body = SuccessResponse),
        (status = 401, description = "Unauthorized", body = AppResponseError),
        (status = 404, description = "Session not found", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
pub async fn close_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> AppResult<Json<SuccessResponse>> {
    let session_manager = state.native_session_manager.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailableError("Session manager not available".to_string())
    })?;

    session_manager.remove_session(&session_id);

    info!("Closed native WebRTC session {}", session_id);

    Ok(Json(SuccessResponse {
        success: true,
        message: "Session closed".to_string(),
    }))
}

/// Get native WebRTC statistics
#[utoipa::path(
    get,
    path = "/api/v3/webrtc/native/stats",
    operation_id = "getNativeWebRtcStats",
    tag = "Native WebRTC",
    responses(
        (status = 200, description = "Native WebRTC statistics", body = NativeWebRtcStatsResponse),
        (status = 401, description = "Unauthorized", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
pub async fn get_stats(State(state): State<AppState>) -> AppResult<Json<NativeWebRtcStatsResponse>> {
    let (active_sessions, engine_status) = if let Some(session_manager) =
        state.native_session_manager.as_ref()
    {
        let count = session_manager.active_session_count();
        let status = if state.native_webrtc_engine.is_some() {
            "available"
        } else {
            "unavailable"
        };
        (count, status.to_string())
    } else {
        (0, "unavailable".to_string())
    };

    Ok(Json(NativeWebRtcStatsResponse {
        active_sessions,
        engine_status,
    }))
}

/// List all active sessions
#[utoipa::path(
    get,
    path = "/api/v3/webrtc/native/sessions",
    operation_id = "listNativeWebRtcSessions",
    tag = "Native WebRTC",
    responses(
        (status = 200, description = "List of active sessions", body = Vec<SessionInfoResponse>),
        (status = 401, description = "Unauthorized", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
pub async fn list_sessions(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<SessionInfoResponse>>> {
    let _session_manager = state.native_session_manager.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailableError("Session manager not available".to_string())
    })?;

    // Return empty list for now - need to implement iteration over DashMap
    Ok(Json(Vec::new()))
}

/// Get session details
#[utoipa::path(
    get,
    path = "/api/v3/webrtc/native/sessions/{session_id}",
    operation_id = "getNativeWebRtcSession",
    tag = "Native WebRTC",
    params(
        ("session_id" = String, Path, description = "Session ID")
    ),
    responses(
        (status = 200, description = "Session details", body = SessionInfoResponse),
        (status = 401, description = "Unauthorized", body = AppResponseError),
        (status = 404, description = "Session not found", body = AppResponseError)
    ),
    security(("jwt" = []))
)]
pub async fn get_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> AppResult<Json<SessionInfoResponse>> {
    let session_manager = state.native_session_manager.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailableError("Session manager not available".to_string())
    })?;

    let stats = session_manager
        .get_session_stats(&session_id)
        .await
        .ok_or_else(|| {
            AppError::NotFoundError(Resource {
                details: vec![("session_id".to_string(), session_id.clone())],
                resource_type: ResourceType::Message,
            })
        })?;

    Ok(Json(SessionInfoResponse {
        session_id: stats.id,
        monitor_id: stats.monitor_id,
        state: stats.state,
        duration_seconds: stats.duration_seconds,
        bytes_sent: stats.bytes_sent,
        packets_sent: stats.packets_sent,
    }))
}

/// Health check for native WebRTC
#[utoipa::path(
    get,
    path = "/api/v3/webrtc/native/health",
    operation_id = "nativeWebRtcHealth",
    tag = "Native WebRTC",
    responses(
        (status = 200, description = "Native WebRTC is healthy", body = SuccessResponse),
        (status = 503, description = "Native WebRTC unavailable", body = AppResponseError)
    )
)]
pub async fn health_check(State(state): State<AppState>) -> AppResult<Json<SuccessResponse>> {
    if state.native_webrtc_engine.is_some() && state.native_session_manager.is_some() {
        Ok(Json(SuccessResponse {
            success: true,
            message: "Native WebRTC engine is healthy".to_string(),
        }))
    } else {
        Err(AppError::ServiceUnavailableError(
            "Native WebRTC engine not available".to_string(),
        ))
    }
}
