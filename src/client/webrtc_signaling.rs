use std::{
    collections::HashMap,
    sync::Arc,
    time::Duration,
};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::{Mutex, RwLock},
    time::timeout,
};
use tracing::{debug, info, error};

/// Message types for communication with the C++ WebRTC plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum PluginMessage {
    /// Create a WebRTC offer for a new viewer session
    CreateOffer {
        camera_id: i32,
        viewer_id: String,
    },
    /// Process the client's SDP answer to complete WebRTC connection
    SetAnswer {
        camera_id: i32,
        viewer_id: String,
        answer: String,
    },
    /// Add ICE candidates for NAT traversal
    AddIceCandidate {
        camera_id: i32,
        viewer_id: String,
        candidate: String,
        sdp_mid: String,
        sdp_mline_index: i32,
    },
    /// Forcefully disconnect a specific viewer
    DropViewer {
        camera_id: i32,
        viewer_id: String,
    },
    /// Get stats for testing/health check
    GetStats {
        camera_id: i32,
        viewer_id: String,
    },
}

/// Response types from the C++ WebRTC plugin  
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PluginResponse {
    /// Success response with SDP offer
    CreateOffer {
        offer: String,
        viewer_id: String,
        camera_id: i32,
        command: String,
        success: bool,
    },
    /// Success response for any other command
    Success {
        viewer_id: Option<String>,
        camera_id: Option<i32>,
        command: String,
        success: bool,
        // Optional fields for stats
        total_frames_processed: Option<i64>,
        total_bytes_processed: Option<i64>,
        total_connections_created: Option<i32>,
        total_connections_dropped: Option<i32>,
        streams: Option<Vec<serde_json::Value>>,
        viewers: Option<Vec<serde_json::Value>>,
    },
    /// Error response
    Error {
        error: String,
        command: Option<String>,
        success: bool,
    },
}

/// WebRTC session information
#[derive(Debug, Clone)]
pub struct WebRtcSession {
    pub camera_id: i32,
    pub viewer_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

/// Session manager for tracking active WebRTC sessions
#[derive(Debug, Clone)]
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, WebRtcSession>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new session
    pub async fn create_session(&self, camera_id: i32, viewer_id: String) -> String {
        let session_key = format!("{}:{}", camera_id, viewer_id);
        let now = chrono::Utc::now();
        
        let session = WebRtcSession {
            camera_id,
            viewer_id: viewer_id.clone(),
            created_at: now,
            last_activity: now,
        };

        self.sessions.write().await.insert(session_key.clone(), session);
        debug!("Created WebRTC session: {}", session_key);
        session_key
    }

    /// Find session key by viewer_id
    pub async fn find_session_by_viewer(&self, viewer_id: &str) -> Option<String> {
        let sessions = self.sessions.read().await;
        for (key, session) in sessions.iter() {
            if session.viewer_id == viewer_id {
                return Some(key.clone());
            }
        }
        None
    }

    /// Update session activity
    pub async fn update_activity(&self, session_key: &str) {
        if let Some(session) = self.sessions.write().await.get_mut(session_key) {
            session.last_activity = chrono::Utc::now();
        }
    }

    /// Remove a session
    pub async fn remove_session(&self, session_key: &str) {
        self.sessions.write().await.remove(session_key);
        debug!("Removed WebRTC session: {}", session_key);
    }

    /// Get active session count
    pub async fn session_count(&self) -> usize {
        self.sessions.read().await.len()
    }

    /// Get all active sessions
    pub async fn list_sessions(&self) -> HashMap<String, WebRtcSession> {
        self.sessions.read().await.clone()
    }

    /// Clean up stale sessions (older than 1 hour)
    pub async fn cleanup_stale_sessions(&self) {
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(1);
        let mut sessions = self.sessions.write().await;
        
        let stale_keys: Vec<String> = sessions
            .iter()
            .filter(|(_, session)| session.last_activity < cutoff)
            .map(|(key, _)| key.clone())
            .collect();

        for key in stale_keys {
            sessions.remove(&key);
            debug!("Cleaned up stale session: {}", key);
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// WebRTC signaling client for communicating with the C++ plugin
#[derive(Debug)]
pub struct WebRtcSignalingClient {
    plugin_address: String,
    session_manager: SessionManager,
    #[allow(dead_code)]
    connection_pool: Arc<Mutex<Option<TcpStream>>>,
}

impl WebRtcSignalingClient {
    /// Create a new WebRTC signaling client
    pub fn new(plugin_address: String) -> Self {
        Self {
            plugin_address,
            session_manager: SessionManager::new(),
            connection_pool: Arc::new(Mutex::new(None)),
        }
    }

    /// Get or create a connection to the plugin
    async fn get_connection(&self) -> Result<TcpStream> {
        // For now, create a new connection each time
        // In production, you might want to implement connection pooling
        let stream = timeout(
            Duration::from_secs(5),
            TcpStream::connect(&self.plugin_address)
        ).await
        .map_err(|_| anyhow!("Connection timeout to plugin at {}", self.plugin_address))?
        .map_err(|e| anyhow!("Failed to connect to plugin: {}", e))?;

        info!("Connected to WebRTC plugin at {}", self.plugin_address);
        Ok(stream)
    }

    /// Send a message to the plugin and wait for response
    async fn send_message(&self, message: PluginMessage) -> Result<PluginResponse> {
        let mut stream = self.get_connection().await?;
        
        // Serialize message to JSON
        let json_message = serde_json::to_string(&message)?;
        let message_with_newline = format!("{}\n", json_message);
        
        info!("Sending to plugin: {}", json_message);
        info!("Message length: {} bytes", message_with_newline.len());
        
        // Send message
        stream.write_all(message_with_newline.as_bytes()).await
            .map_err(|e| anyhow!("Failed to send message: {}", e))?;

        info!("Message sent successfully, waiting for response...");

        // Read response with timeout
        let mut reader = BufReader::new(stream);
        let mut response_line = String::new();
        
        let bytes_read = timeout(
            Duration::from_secs(10),
            reader.read_line(&mut response_line)
        ).await
        .map_err(|_| anyhow!("Response timeout from plugin"))?
        .map_err(|e| anyhow!("Failed to read response: {}", e))?;

        if bytes_read == 0 {
            return Err(anyhow!("Plugin closed connection"));
        }

        // Remove trailing newline
        response_line = response_line.trim().to_string();
        info!("Raw response from plugin: '{}'", response_line);
        info!("Response length: {} bytes", response_line.len());

        // Log the raw bytes for debugging
        let response_bytes: Vec<u8> = response_line.bytes().collect();
        info!("Response bytes: {:?}", response_bytes);

        // Parse response
        let response: PluginResponse = serde_json::from_str(&response_line)
            .map_err(|e| {
                error!("Failed to parse plugin response. Raw response: '{}'", response_line);
                error!("Parse error: {}", e);
                anyhow!("Failed to parse plugin response: {}", e)
            })?;

        Ok(response)
    }

    /// Request an SDP offer for a camera stream
    pub async fn get_offer(&self, camera_id: i32, viewer_id: String) -> Result<String> {
        info!("Creating WebRTC offer for camera_id: {}, viewer_id: {}", camera_id, viewer_id);
        
        let session_key = self.session_manager.create_session(camera_id, viewer_id.clone()).await;
        info!("Created session with key: {}", session_key);
        
        let message = PluginMessage::CreateOffer {
            camera_id,
            viewer_id: viewer_id.clone(),
        };

        info!("Sending create_offer message: {:?}", message);
        
        match self.send_message(message).await? {
            PluginResponse::CreateOffer { offer, success, .. } => {
                info!("Received CreateOffer response: success={}, offer length={}", success, offer.len());
                if success {
                    self.session_manager.update_activity(&session_key).await;
                    Ok(offer)
                } else {
                    self.session_manager.remove_session(&session_key).await;
                    Err(anyhow!("Failed to create offer"))
                }
            }
            PluginResponse::Error { error, .. } => {
                self.session_manager.remove_session(&session_key).await;
                Err(anyhow!("Plugin error: {}", error))
            }
            _ => {
                self.session_manager.remove_session(&session_key).await;
                Err(anyhow!("Unexpected response type for offer request"))
            }
        }
    }

    /// Send SDP answer from viewer
    pub async fn send_answer(&self, camera_id: i32, viewer_id: String, answer: String) -> Result<bool> {
        let message = PluginMessage::SetAnswer {
            camera_id,
            viewer_id: viewer_id.clone(),
            answer,
        };

        match self.send_message(message).await? {
            PluginResponse::Success { success, command, .. } if command == "set_answer" => {
                if success {
                    // Find session by viewer_id and update activity
                    if let Some(session_key) = self.session_manager.find_session_by_viewer(&viewer_id).await {
                        self.session_manager.update_activity(&session_key).await;
                    }
                }
                Ok(success)
            }
            PluginResponse::Error { error, .. } => {
                Err(anyhow!("Plugin error: {}", error))
            }
            _ => Err(anyhow!("Unexpected response type for answer request")),
        }
    }

    /// Forward ICE candidate
    pub async fn send_candidate(&self, camera_id: i32, viewer_id: String, candidate: String, sdp_mid: String, sdp_mline_index: i32) -> Result<bool> {
        let message = PluginMessage::AddIceCandidate {
            camera_id,
            viewer_id: viewer_id.clone(),
            candidate,
            sdp_mid,
            sdp_mline_index,
        };

        match self.send_message(message).await? {
            PluginResponse::Success { success, command, .. } if command == "add_ice_candidate" => {
                if success {
                    // Find session by viewer_id and update activity
                    if let Some(session_key) = self.session_manager.find_session_by_viewer(&viewer_id).await {
                        self.session_manager.update_activity(&session_key).await;
                    }
                }
                Ok(success)
            }
            PluginResponse::Error { error, .. } => {
                Err(anyhow!("Plugin error: {}", error))
            }
            _ => Err(anyhow!("Unexpected response type for candidate")),
        }
    }

    /// Drop a viewer connection
    pub async fn drop_viewer(&self, camera_id: i32, viewer_id: String) -> Result<bool> {
        let message = PluginMessage::DropViewer {
            camera_id,
            viewer_id: viewer_id.clone(),
        };

        let result = match self.send_message(message).await? {
            PluginResponse::Success { success, command, .. } if command == "drop_viewer" => Ok(success),
            PluginResponse::Error { error, .. } => {
                Err(anyhow!("Plugin error: {}", error))
            }
            _ => Err(anyhow!("Unexpected response type for drop viewer")),
        };

        // Always remove from session manager
        if let Some(session_key) = self.session_manager.find_session_by_viewer(&viewer_id).await {
            self.session_manager.remove_session(&session_key).await;
        }
        result
    }

    /// Get session manager
    pub fn sessions(&self) -> &SessionManager {
        &self.session_manager
    }

    /// Test connection to the plugin with get_stats
    pub async fn test_connection(&self) -> Result<()> {
        let message = PluginMessage::GetStats {
            camera_id: 1,
            viewer_id: "test".to_string(),
        };
        
        match self.send_message(message).await? {
            PluginResponse::Success { success, command, .. } if command == "get_stats" => {
                if success {
                    info!("WebRTC plugin connection test successful");
                    Ok(())
                } else {
                    Err(anyhow!("Plugin stats request failed"))
                }
            }
            PluginResponse::Error { error, .. } => {
                Err(anyhow!("Plugin error: {}", error))
            }
            _ => Err(anyhow!("Unexpected response to stats request")),
        }
    }
}

impl Clone for WebRtcSignalingClient {
    fn clone(&self) -> Self {
        Self {
            plugin_address: self.plugin_address.clone(),
            session_manager: self.session_manager.clone(),
            connection_pool: Arc::new(Mutex::new(None)),
        }
    }
}
