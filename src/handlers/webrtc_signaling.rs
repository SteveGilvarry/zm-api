use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use utoipa::ToSchema;

use crate::{
    error::{AppError, AppResponseError, AppResult},
    server::state::AppState,
};

/// Request body for SDP answer
#[derive(Debug, Deserialize, ToSchema)]
pub struct AnswerRequest {
    pub answer: String,
}

/// Request body for ICE candidate
#[derive(Debug, Deserialize, ToSchema)]
pub struct CandidateRequest {
    pub candidate: String,
    pub sdp_mid: String,
    pub sdp_mline_index: i32,
}

/// Response for SDP offer
#[derive(Debug, Serialize, ToSchema)]
pub struct OfferResponse {
    pub offer: String,
    pub viewer_id: String,
}

/// Success response for operations
#[derive(Debug, Serialize, ToSchema)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: String,
}

/// Get WebRTC SDP offer for a camera stream
#[utoipa::path(
    get,
    path = "/api/v3/streaming/webrtc/{camera_id}/{viewer_id}/offer",
    operation_id = "getWebRtcOffer",
    tag = "WebRTC Streaming",
    params(
        ("camera_id" = i32, Path, description = "Camera/Monitor ID"),
        ("viewer_id" = String, Path, description = "Unique viewer ID")
    ),
    responses(
        (status = 200, description = "SDP offer generated successfully", body = OfferResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 401, description = "Unauthorized", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError),
        (status = 503, description = "WebRTC plugin unavailable", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn get_webrtc_offer(
    State(state): State<AppState>,
    Path((camera_id, viewer_id)): Path<(i32, String)>,
) -> AppResult<Json<OfferResponse>> {
    info!(
        "Requesting WebRTC offer for camera {} and viewer {}",
        camera_id, viewer_id
    );

    // Validate viewer_id
    if viewer_id.is_empty() {
        return Err(AppError::BadRequestError(
            "Viewer ID must not be empty".to_string(),
        ));
    }

    // Get WebRTC client from app state
    let webrtc_client = &state.webrtc_client;

    // Request SDP offer from plugin
    match webrtc_client.get_offer(camera_id, viewer_id.clone()).await {
        Ok(offer) => {
            info!(
                "Successfully generated WebRTC offer for camera {} and viewer {}",
                camera_id, viewer_id
            );
            Ok(Json(OfferResponse { offer, viewer_id }))
        }
        Err(e) => {
            error!(
                "Failed to get WebRTC offer for camera {} and viewer {}: {}",
                camera_id, viewer_id, e
            );
            Err(AppError::InternalServerError(format!(
                "WebRTC plugin error: {}",
                e
            )))
        }
    }
}

/// Send WebRTC SDP answer from viewer
#[utoipa::path(
    post,
    path = "/api/v3/streaming/webrtc/{camera_id}/{viewer_id}/answer",
    operation_id = "sendWebRtcAnswer",
    tag = "WebRTC Streaming",
    params(
        ("camera_id" = i32, Path, description = "Camera/Monitor ID"),
        ("viewer_id" = String, Path, description = "Unique viewer ID")
    ),
    request_body = AnswerRequest,
    responses(
        (status = 200, description = "SDP answer processed successfully", body = SuccessResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 401, description = "Unauthorized", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError),
        (status = 503, description = "WebRTC plugin unavailable", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn send_webrtc_answer(
    State(state): State<AppState>,
    Path((camera_id, viewer_id)): Path<(i32, String)>,
    Json(answer_req): Json<AnswerRequest>,
) -> AppResult<Json<SuccessResponse>> {
    info!(
        "Sending WebRTC answer for viewer {} on camera {}",
        viewer_id, camera_id
    );

    // Validate inputs
    if viewer_id.is_empty() {
        return Err(AppError::BadRequestError(
            "Viewer ID must not be empty".to_string(),
        ));
    }

    if answer_req.answer.is_empty() {
        return Err(AppError::BadRequestError(
            "SDP answer must not be empty".to_string(),
        ));
    }

    // Get WebRTC client from app state
    let webrtc_client = &state.webrtc_client;

    // Send SDP answer to plugin
    match webrtc_client
        .send_answer(camera_id, viewer_id.clone(), answer_req.answer)
        .await
    {
        Ok(success) => {
            if success {
                info!(
                    "Successfully processed WebRTC answer for viewer {}",
                    viewer_id
                );
                Ok(Json(SuccessResponse {
                    success: true,
                    message: "SDP answer processed successfully".to_string(),
                }))
            } else {
                warn!("WebRTC plugin rejected answer for viewer {}", viewer_id);
                Err(AppError::BadRequestError(
                    "WebRTC plugin rejected the SDP answer".to_string(),
                ))
            }
        }
        Err(e) => {
            error!(
                "Failed to send WebRTC answer for viewer {}: {}",
                viewer_id, e
            );
            Err(AppError::InternalServerError(format!(
                "WebRTC plugin error: {}",
                e
            )))
        }
    }
}

/// Forward ICE candidate from viewer
#[utoipa::path(
    post,
    path = "/api/v3/streaming/webrtc/{camera_id}/{viewer_id}/candidate",
    operation_id = "sendWebRtcCandidate",
    tag = "WebRTC Streaming",
    params(
        ("camera_id" = i32, Path, description = "Camera/Monitor ID"),
        ("viewer_id" = String, Path, description = "Unique viewer ID")
    ),
    request_body = CandidateRequest,
    responses(
        (status = 200, description = "ICE candidate processed successfully", body = SuccessResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 401, description = "Unauthorized", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError),
        (status = 503, description = "WebRTC plugin unavailable", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn send_webrtc_candidate(
    State(state): State<AppState>,
    Path((camera_id, viewer_id)): Path<(i32, String)>,
    Json(candidate_req): Json<CandidateRequest>,
) -> AppResult<Json<SuccessResponse>> {
    info!(
        "Sending WebRTC ICE candidate for viewer {} on camera {}",
        viewer_id, camera_id
    );

    // Validate inputs
    if viewer_id.is_empty() {
        return Err(AppError::BadRequestError(
            "Viewer ID must not be empty".to_string(),
        ));
    }

    if candidate_req.candidate.is_empty() {
        return Err(AppError::BadRequestError(
            "ICE candidate must not be empty".to_string(),
        ));
    }

    // Get WebRTC client from app state
    let webrtc_client = &state.webrtc_client;

    // Send ICE candidate to plugin
    match webrtc_client
        .send_candidate(
            camera_id,
            viewer_id.clone(),
            candidate_req.candidate,
            candidate_req.sdp_mid,
            candidate_req.sdp_mline_index,
        )
        .await
    {
        Ok(success) => {
            if success {
                info!(
                    "Successfully processed WebRTC ICE candidate for viewer {}",
                    viewer_id
                );
                Ok(Json(SuccessResponse {
                    success: true,
                    message: "ICE candidate processed successfully".to_string(),
                }))
            } else {
                warn!(
                    "WebRTC plugin rejected ICE candidate for viewer {}",
                    viewer_id
                );
                Err(AppError::BadRequestError(
                    "WebRTC plugin rejected the ICE candidate".to_string(),
                ))
            }
        }
        Err(e) => {
            error!(
                "Failed to send WebRTC ICE candidate for viewer {}: {}",
                viewer_id, e
            );
            Err(AppError::InternalServerError(format!(
                "WebRTC plugin error: {}",
                e
            )))
        }
    }
}

/// Forcibly drop a WebRTC viewer connection
#[utoipa::path(
    delete,
    path = "/api/v3/streaming/webrtc/{viewer_id}",
    operation_id = "dropWebRtcViewer",
    tag = "WebRTC Streaming",
    params(
        ("viewer_id" = String, Path, description = "Unique viewer ID")
    ),
    responses(
        (status = 200, description = "Viewer connection dropped successfully", body = SuccessResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 401, description = "Unauthorized", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError),
        (status = 503, description = "WebRTC plugin unavailable", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn drop_webrtc_viewer(
    State(state): State<AppState>,
    Path(viewer_id): Path<String>,
) -> AppResult<Json<SuccessResponse>> {
    info!("Dropping WebRTC viewer connection for viewer {}", viewer_id);

    // Validate inputs
    if viewer_id.is_empty() {
        return Err(AppError::BadRequestError(
            "Viewer ID must not be empty".to_string(),
        ));
    }

    // Get WebRTC client from app state
    let webrtc_client = &state.webrtc_client;

    // Find the camera_id for this viewer from our session manager
    let camera_id = if let Some(session_key) = webrtc_client
        .sessions()
        .find_session_by_viewer(&viewer_id)
        .await
    {
        let sessions = webrtc_client.sessions().list_sessions().await;
        if let Some(session) = sessions.get(&session_key) {
            session.camera_id
        } else {
            // Session not found, just clean up locally
            warn!(
                "Session not found for viewer {}, cleaning up locally",
                viewer_id
            );
            return Ok(Json(SuccessResponse {
                success: true,
                message: "Viewer session cleaned up locally".to_string(),
            }));
        }
    } else {
        // No session found, already cleaned up
        return Ok(Json(SuccessResponse {
            success: true,
            message: "Viewer session already cleaned up".to_string(),
        }));
    };

    // Drop viewer connection
    match webrtc_client
        .drop_viewer(camera_id, viewer_id.clone())
        .await
    {
        Ok(success) => {
            if success {
                info!(
                    "Successfully dropped WebRTC viewer connection for viewer {}",
                    viewer_id
                );
                Ok(Json(SuccessResponse {
                    success: true,
                    message: "Viewer connection dropped successfully".to_string(),
                }))
            } else {
                warn!(
                    "WebRTC plugin reported failure dropping viewer {}",
                    viewer_id
                );
                // Still return success since the session is cleaned up locally
                Ok(Json(SuccessResponse {
                    success: true,
                    message: "Viewer connection cleaned up locally".to_string(),
                }))
            }
        }
        Err(e) => {
            error!("Failed to drop WebRTC viewer {}: {}", viewer_id, e);
            // Clean up locally and return success
            Ok(Json(SuccessResponse {
                success: true,
                message: "Viewer connection cleaned up locally due to plugin error".to_string(),
            }))
        }
    }
}

/// Get WebRTC session statistics (for debugging/monitoring)
#[utoipa::path(
    get,
    path = "/api/v3/streaming/sessions",
    operation_id = "getWebRtcSessions",
    tag = "WebRTC Streaming",
    responses(
        (status = 200, description = "Session statistics retrieved successfully"),
        (status = 401, description = "Unauthorized", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn get_webrtc_sessions(
    State(state): State<AppState>,
) -> AppResult<Json<serde_json::Value>> {
    let webrtc_client = &state.webrtc_client;
    let session_count = webrtc_client.sessions().session_count().await;

    Ok(Json(serde_json::json!({
        "active_sessions": session_count,
        "plugin_address": "127.0.0.1:9050"
    })))
}

/// Test connection to WebRTC plugin (for health checks)
#[utoipa::path(
    get,
    path = "/api/v3/streaming/health",
    operation_id = "testWebRtcConnection",
    tag = "WebRTC Streaming",
    responses(
        (status = 200, description = "WebRTC plugin is healthy", body = SuccessResponse),
        (status = 503, description = "WebRTC plugin unavailable", body = AppResponseError)
    )
)]
pub async fn test_webrtc_connection(
    State(state): State<AppState>,
) -> AppResult<Json<SuccessResponse>> {
    let webrtc_client = &state.webrtc_client;

    match webrtc_client.test_connection().await {
        Ok(_) => Ok(Json(SuccessResponse {
            success: true,
            message: "WebRTC plugin is healthy".to_string(),
        })),
        Err(e) => {
            error!("WebRTC plugin health check failed: {}", e);
            Err(AppError::ServiceUnavailableError(format!(
                "WebRTC plugin unavailable: {}",
                e
            )))
        }
    }
}
