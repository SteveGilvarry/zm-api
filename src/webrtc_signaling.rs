use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{mpsc, RwLock};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug};
use axum::extract::ws::{WebSocket, Message};
use futures_util::{SinkExt, StreamExt};
use uuid::Uuid;
use tokio::time::{interval, Duration};

use crate::webrtc_ffi::{
    WebRTCSession, SessionDescription, IceCandidate, SessionState, 
    list_camera_streams, get_webrtc_stats, create_webrtc_client,
    get_client_connection_state, remove_webrtc_client, set_webrtc_offer,
    add_webrtc_ice_candidate, CameraStream
};

/// WebSocket message types for WebRTC signaling
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum SignalingMessage {
    /// Client requests to join a stream
    JoinStream { stream_id: String },
    /// Client leaves the stream
    LeaveStream,
    /// WebRTC offer from client
    Offer { 
        offer: SessionDescription,
        stream_id: String,
    },
    /// WebRTC answer from server
    Answer { answer: SessionDescription },
    /// ICE candidate exchange
    IceCandidate { candidate: IceCandidate },
    /// Request list of available streams
    ListStreams,
    /// Response with available streams
    StreamList { streams: Vec<StreamInfo> },
    /// Connection state change notification
    StateChange { 
        state: SessionState,
        stream_id: String,
    },
    /// Error message
    Error { message: String },
    /// Success/acknowledgment message
    Success { message: String },
    /// Service statistics
    Stats {
        total_frames: u64,
        total_bytes: u64,
        clients_connected: u64,
        clients_disconnected: u64,
        active_streams: usize,
    },
    /// Ping/pong for connection keepalive
    Ping,
    Pong,
}

/// Stream information for client discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub active: bool,
    pub client_count: i32,
    pub resolution: String,
}

/// Client connection information
#[derive(Debug)]
pub struct ClientConnection {
    pub id: String,
    pub stream_id: Option<String>,
    pub sender: mpsc::UnboundedSender<SignalingMessage>,
    pub state: SessionState,
    pub webrtc_client_id: Option<String>,
}

/// WebRTC signaling service - acts as the signaling layer for the C++ WebRTC service
pub struct WebRTCSignalingService {
    clients: Arc<RwLock<HashMap<String, ClientConnection>>>,
    stream_clients: Arc<RwLock<HashMap<String, Vec<String>>>>, // stream_id -> client_ids
    service_stats_interval: Duration,
}

impl WebRTCSignalingService {
    /// Create a new signaling service (connects to existing C++ WebRTC service)
    pub fn new() -> Result<Self> {
        info!("Initializing WebRTC signaling service...");
        
        // Test connection to the C++ WebRTC service
        match get_webrtc_stats() {
            Ok(stats) => {
                info!("✅ Connected to WebRTC service - Frames: {}, Bytes: {}", 
                      stats.total_frames, stats.total_bytes);
            }
            Err(e) => {
                warn!("⚠️  WebRTC service connection issue: {}", e);
                return Err(anyhow!("Failed to connect to WebRTC service: {}", e));
            }
        }
        
        Ok(Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            stream_clients: Arc::new(RwLock::new(HashMap::new())),
            service_stats_interval: Duration::from_secs(30),
        })
    }
    
    /// Start the signaling service background tasks
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting WebRTC signaling service...");
        
        // Start periodic tasks
        self.start_stats_broadcaster().await;
        self.start_stream_monitor().await;
        
        Ok(())
    }
    
    /// Discover available camera streams from the C++ service
    pub async fn discover_streams(&self) -> Result<Vec<StreamInfo>> {
        match list_camera_streams() {
            Ok(camera_streams) => {
                let mut streams = Vec::new();
                let clients = self.clients.read().await;
                
                for camera in camera_streams {
                    let client_count = self.count_clients_for_stream(&camera.stream_id, &clients).await;
                    
                    streams.push(StreamInfo {
                        id: camera.stream_id,
                        name: camera.camera_name,
                        description: Some(format!("Camera stream - {}", camera.resolution)),
                        active: camera.is_active,
                        client_count,
                        resolution: camera.resolution,
                    });
                }
                
                Ok(streams)
            }
            Err(e) => {
                warn!("Failed to discover camera streams: {}", e);
                Ok(vec![]) // Return empty list on error
            }
        }
    }
    
    /// Handle a new WebSocket connection
    pub async fn handle_connection(&self, socket: WebSocket) -> Result<()> {
        let client_id = format!("client_{}", Uuid::new_v4());
        info!("🔌 New WebSocket connection: {}", client_id);
        
        let (mut ws_sender, mut ws_receiver) = socket.split();
        let (tx, mut rx) = mpsc::unbounded_channel();
        
        // Add client to our tracking
        {
            let mut clients = self.clients.write().await;
            clients.insert(client_id.clone(), ClientConnection {
                id: client_id.clone(),
                stream_id: None,
                sender: tx.clone(),
                state: SessionState::New,
                webrtc_client_id: None,
            });
        }
        
        // Send welcome message with available streams
        let streams = self.discover_streams().await?;
        let welcome_msg = SignalingMessage::StreamList { streams };
        if let Err(e) = tx.send(welcome_msg) {
            warn!("Failed to send welcome message: {}", e);
        }
        
        // Clone necessary data for the tasks
        let clients_clone = Arc::clone(&self.clients);
        let stream_clients_clone = Arc::clone(&self.stream_clients);
        let client_id_clone = client_id.clone();
        
        // Task to send messages from our channel to WebSocket
        let send_task = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                let ws_msg = match serde_json::to_string(&msg) {
                    Ok(json) => Message::Text(json),
                    Err(e) => {
                        error!("Failed to serialize message: {}", e);
                        continue;
                    }
                };
                
                if let Err(e) = ws_sender.send(ws_msg).await {
                    debug!("WebSocket send failed: {}", e);
                    break;
                }
            }
        });
        
        // Task to receive messages from WebSocket
        let client_id_for_recv = client_id.clone();
        let recv_task = tokio::spawn(async move {
            while let Some(msg_result) = ws_receiver.next().await {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        if let Err(e) = Self::handle_message(
                            &text, 
                            &client_id_for_recv, 
                            &clients_clone, 
                            &stream_clients_clone
                        ).await {
                            error!("Error handling message: {}", e);
                        }
                    }
                    Ok(Message::Close(_)) => {
                        info!("Client {} disconnected", client_id_for_recv);
                        break;
                    }
                    Ok(Message::Ping(data)) => {
                        debug!("Ping from client {}", client_id_for_recv);
                    }
                    Err(e) => {
                        debug!("WebSocket error for client {}: {}", client_id_for_recv, e);
                        break;
                    }
                    _ => {} // Ignore other message types
                }
            }
        });
        
        // Wait for either task to complete (indicating disconnection)
        tokio::select! {
            _ = send_task => {},
            _ = recv_task => {},
        }
        
        // Cleanup client
        self.cleanup_client(&client_id).await;
        info!("🔌 Client {} disconnected and cleaned up", client_id);
        
        Ok(())
    }
    
    /// Handle an incoming signaling message
    async fn handle_message(
        text: &str,
        client_id: &str,
        clients: &Arc<RwLock<HashMap<String, ClientConnection>>>,
        stream_clients: &Arc<RwLock<HashMap<String, Vec<String>>>>
    ) -> Result<()> {
        debug!("Received message from {}: {}", client_id, text);
        
        let msg: SignalingMessage = serde_json::from_str(text)?;
        
        match msg {
            SignalingMessage::JoinStream { stream_id } => {
                Self::handle_join_stream(client_id, &stream_id, clients, stream_clients).await?;
            }
            SignalingMessage::LeaveStream => {
                Self::handle_leave_stream(client_id, clients, stream_clients).await?;
            }
            SignalingMessage::Offer { offer, stream_id } => {
                Self::handle_offer(client_id, offer, &stream_id, clients).await?;
            }
            SignalingMessage::IceCandidate { candidate } => {
                Self::handle_ice_candidate(client_id, candidate, clients).await?;
            }
            SignalingMessage::ListStreams => {
                Self::handle_list_streams(client_id, clients).await?;
            }
            SignalingMessage::Ping => {
                Self::send_to_client(client_id, SignalingMessage::Pong, clients).await?;
            }
            _ => {
                warn!("Unhandled message type from client {}", client_id);
            }
        }
        
        Ok(())
    }
    
    /// Handle client joining a stream
    async fn handle_join_stream(
        client_id: &str,
        stream_id: &str,
        clients: &Arc<RwLock<HashMap<String, ClientConnection>>>,
        stream_clients: &Arc<RwLock<HashMap<String, Vec<String>>>>
    ) -> Result<()> {
        info!("Client {} joining stream {}", client_id, stream_id);
        
        // Create WebRTC client in the C++ service
        let webrtc_client_id = match create_webrtc_client(stream_id) {
            Ok(id) => id,
            Err(e) => {
                let error_msg = format!("Failed to create WebRTC client: {}", e);
                Self::send_to_client(client_id, SignalingMessage::Error { message: error_msg }, clients).await?;
                return Ok(());
            }
        };
        
        // Update client connection
        {
            let mut clients_map = clients.write().await;
            if let Some(client) = clients_map.get_mut(client_id) {
                client.stream_id = Some(stream_id.to_string());
                client.webrtc_client_id = Some(webrtc_client_id.clone());
                client.state = SessionState::Connecting;
            }
        }
        
        // Add to stream clients tracking
        {
            let mut stream_clients_map = stream_clients.write().await;
            stream_clients_map
                .entry(stream_id.to_string())
                .or_insert_with(Vec::new)
                .push(client_id.to_string());
        }
        
        // Send success message
        Self::send_to_client(
            client_id,
            SignalingMessage::Success { 
                message: format!("Joined stream {} successfully", stream_id) 
            },
            clients
        ).await?;
        
        Ok(())
    }
    
    /// Handle client leaving a stream
    async fn handle_leave_stream(
        client_id: &str,
        clients: &Arc<RwLock<HashMap<String, ClientConnection>>>,
        stream_clients: &Arc<RwLock<HashMap<String, Vec<String>>>>
    ) -> Result<()> {
        info!("Client {} leaving stream", client_id);
        
        let (stream_id, webrtc_client_id) = {
            let mut clients_map = clients.write().await;
            if let Some(client) = clients_map.get_mut(client_id) {
                let stream_id = client.stream_id.take();
                let webrtc_client_id = client.webrtc_client_id.take();
                client.state = SessionState::Disconnected;
                (stream_id, webrtc_client_id)
            } else {
                return Ok(());
            }
        };
        
        // Remove from stream clients tracking
        if let Some(stream_id) = &stream_id {
            let mut stream_clients_map = stream_clients.write().await;
            if let Some(clients_list) = stream_clients_map.get_mut(stream_id) {
                clients_list.retain(|id| id != client_id);
                if clients_list.is_empty() {
                    stream_clients_map.remove(stream_id);
                }
            }
        }
        
        // Remove WebRTC client from C++ service
        if let Some(webrtc_client_id) = webrtc_client_id {
            if let Err(e) = remove_webrtc_client(&webrtc_client_id) {
                warn!("Failed to remove WebRTC client {}: {}", webrtc_client_id, e);
            }
        }
        
        Ok(())
    }
    
    /// Handle WebRTC offer from client
    async fn handle_offer(
        client_id: &str,
        offer: SessionDescription,
        stream_id: &str,
        clients: &Arc<RwLock<HashMap<String, ClientConnection>>>
    ) -> Result<()> {
        info!("Handling WebRTC offer from client {} for stream {}", client_id, stream_id);
        
        let webrtc_client_id = {
            let clients_map = clients.read().await;
            if let Some(client) = clients_map.get(client_id) {
                client.webrtc_client_id.clone()
            } else {
                return Err(anyhow!("Client {} not found", client_id));
            }
        };
        
        if let Some(webrtc_client_id) = webrtc_client_id {
            // Set offer in C++ WebRTC service
            match set_webrtc_offer(&webrtc_client_id, &offer) {
                Ok(answer) => {
                    // Send answer back to client
                    Self::send_to_client(
                        client_id,
                        SignalingMessage::Answer { answer },
                        clients
                    ).await?;
                    
                    // Update client state
                    {
                        let mut clients_map = clients.write().await;
                        if let Some(client) = clients_map.get_mut(client_id) {
                            client.state = SessionState::Connected;
                        }
                    }
                }
                Err(e) => {
                    let error_msg = format!("Failed to process WebRTC offer: {}", e);
                    Self::send_to_client(
                        client_id,
                        SignalingMessage::Error { message: error_msg },
                        clients
                    ).await?;
                }
            }
        } else {
            Self::send_to_client(
                client_id,
                SignalingMessage::Error { 
                    message: "No WebRTC client associated with this connection".to_string() 
                },
                clients
            ).await?;
        }
        
        Ok(())
    }
    
    /// Handle ICE candidate from client
    async fn handle_ice_candidate(
        client_id: &str,
        candidate: IceCandidate,
        clients: &Arc<RwLock<HashMap<String, ClientConnection>>>
    ) -> Result<()> {
        debug!("Handling ICE candidate from client {}", client_id);
        
        let webrtc_client_id = {
            let clients_map = clients.read().await;
            if let Some(client) = clients_map.get(client_id) {
                client.webrtc_client_id.clone()
            } else {
                return Err(anyhow!("Client {} not found", client_id));
            }
        };
        
        if let Some(webrtc_client_id) = webrtc_client_id {
            if let Err(e) = add_webrtc_ice_candidate(&webrtc_client_id, &candidate) {
                warn!("Failed to add ICE candidate for client {}: {}", client_id, e);
            }
        }
        
        Ok(())
    }
    
    /// Handle request for stream list
    async fn handle_list_streams(
        client_id: &str,
        clients: &Arc<RwLock<HashMap<String, ClientConnection>>>
    ) -> Result<()> {
        debug!("Client {} requesting stream list", client_id);
        
        let streams = match list_camera_streams() {
            Ok(camera_streams) => {
                camera_streams.into_iter().map(|camera| StreamInfo {
                    id: camera.stream_id,
                    name: camera.camera_name,
                    description: Some(format!("Camera stream - {}", camera.resolution)),
                    active: camera.is_active,
                    client_count: 0, // TODO: Get actual count
                    resolution: camera.resolution,
                }).collect()
            }
            Err(_) => vec![]
        };
        
        Self::send_to_client(
            client_id,
            SignalingMessage::StreamList { streams },
            clients
        ).await?;
        
        Ok(())
    }
    
    /// Send a message to a specific client
    async fn send_to_client(
        client_id: &str,
        message: SignalingMessage,
        clients: &Arc<RwLock<HashMap<String, ClientConnection>>>
    ) -> Result<()> {
        let clients_map = clients.read().await;
        if let Some(client) = clients_map.get(client_id) {
            if let Err(e) = client.sender.send(message) {
                warn!("Failed to send message to client {}: {}", client_id, e);
            }
        }
        Ok(())
    }
    
    /// Clean up a disconnected client
    async fn cleanup_client(&self, client_id: &str) {
        info!("Cleaning up client {}", client_id);
        
        // Remove from clients map and get connection info
        let (stream_id, webrtc_client_id) = {
            let mut clients = self.clients.write().await;
            if let Some(client) = clients.remove(client_id) {
                (client.stream_id, client.webrtc_client_id)
            } else {
                return;
            }
        };
        
        // Remove from stream clients tracking
        if let Some(stream_id) = &stream_id {
            let mut stream_clients = self.stream_clients.write().await;
            if let Some(clients_list) = stream_clients.get_mut(stream_id) {
                clients_list.retain(|id| id != client_id);
                if clients_list.is_empty() {
                    stream_clients.remove(stream_id);
                }
            }
        }
        
        // Remove WebRTC client from C++ service
        if let Some(webrtc_client_id) = webrtc_client_id {
            if let Err(e) = remove_webrtc_client(&webrtc_client_id) {
                warn!("Failed to remove WebRTC client {}: {}", webrtc_client_id, e);
            }
        }
    }
    
    /// Count clients for a specific stream
    async fn count_clients_for_stream(
        &self,
        stream_id: &str,
        clients: &HashMap<String, ClientConnection>
    ) -> i32 {
        clients.values()
            .filter(|client| client.stream_id.as_ref() == Some(&stream_id.to_string()))
            .count() as i32
    }
    
    /// Start background task to broadcast statistics
    async fn start_stats_broadcaster(&self) {
        let clients = Arc::clone(&self.clients);
        let interval_duration = self.service_stats_interval;
        
        tokio::spawn(async move {
            let mut interval = interval(interval_duration);
            
            loop {
                interval.tick().await;
                
                if let Ok(stats) = get_webrtc_stats() {
                    let clients_map = clients.read().await;
                    let active_streams = clients_map.values()
                        .filter_map(|client| client.stream_id.as_ref())
                        .collect::<std::collections::HashSet<_>>()
                        .len();
                    
                    let stats_msg = SignalingMessage::Stats {
                        total_frames: stats.total_frames,
                        total_bytes: stats.total_bytes,
                        clients_connected: stats.clients_connected,
                        clients_disconnected: stats.clients_disconnected,
                        active_streams,
                    };
                    
                    // Broadcast to all clients
                    for client in clients_map.values() {
                        if let Err(e) = client.sender.send(stats_msg.clone()) {
                            debug!("Failed to send stats to client {}: {}", client.id, e);
                        }
                    }
                } else {
                    debug!("Failed to get WebRTC stats for broadcasting");
                }
            }
        });
    }
    
    /// Start background task to monitor streams
    async fn start_stream_monitor(&self) {
        let clients = Arc::clone(&self.clients);
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));
            
            loop {
                interval.tick().await;
                
                // Monitor client connections and update states
                let clients_map = clients.read().await;
                for client in clients_map.values() {
                    if let Some(webrtc_client_id) = &client.webrtc_client_id {
                        if let Ok(state) = get_client_connection_state(webrtc_client_id) {
                            // We could update the client state here if needed
                            debug!("Client {} WebRTC state: {:?}", client.id, state);
                        }
                    }
                }
            }
        });
    }
    
    /// Get current service statistics
    pub async fn get_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        
        let clients = self.clients.read().await;
        stats.insert("total_clients".to_string(), serde_json::Value::Number(clients.len().into()));
        
        let active_streams = clients.values()
            .filter_map(|client| client.stream_id.as_ref())
            .collect::<std::collections::HashSet<_>>()
            .len();
        stats.insert("active_streams".to_string(), serde_json::Value::Number(active_streams.into()));
        
        if let Ok(webrtc_stats) = get_webrtc_stats() {
            stats.insert("webrtc_frames".to_string(), serde_json::Value::Number(webrtc_stats.total_frames.into()));
            stats.insert("webrtc_bytes".to_string(), serde_json::Value::Number(webrtc_stats.total_bytes.into()));
        }
        
        stats
    }
    
    /// Get current client count
    pub async fn get_client_count(&self) -> usize {
        let clients = self.clients.read().await;
        clients.len()
    }
}

impl Clone for WebRTCSignalingService {
    fn clone(&self) -> Self {
        Self {
            clients: Arc::clone(&self.clients),
            stream_clients: Arc::clone(&self.stream_clients),
            service_stats_interval: self.service_stats_interval,
        }
    }
}
