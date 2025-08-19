use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};

use crate::mse_socket_client::MseSocketClient;

/// Error types for MSE operations
#[derive(Debug, thiserror::Error)]
pub enum MseError {
    #[error("Stream registration failed for camera {0}")]
    RegistrationFailed(u32),
    #[error("No segment available")]
    NoSegmentAvailable,
    #[error("Buffer too small: need {needed}, got {available}")]
    BufferTooSmall { needed: usize, available: usize },
    #[error("Stream not found for camera {0}")]
    StreamNotFound(u32),
    #[error("Socket error: {0}")]
    SocketError(String),
}

/// Statistics for an MSE stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MseStats {
    pub camera_id: u32,
    pub buffer_size: usize,
    pub total_segments: u64,
    pub dropped_segments: u64,
    pub bytes_received: u64,
    pub frame_count: u64,
    pub active_clients: usize,
    pub last_segment_time: u64,
}

/// A single fMP4 segment
#[derive(Debug, Clone)]
pub struct Segment {
    pub data: Vec<u8>,
    pub sequence: u64,
    pub timestamp: u64,
    pub duration_ms: u32,
    pub is_init: bool,
}

/// Manages segments for a single camera stream
#[derive(Debug)]
pub struct SegmentManager {
    segments: VecDeque<Segment>,
    init_segment: Option<Segment>,
    sequence_number: u64,
    max_segments: usize,
    camera_id: u32,
}

impl SegmentManager {
    pub fn new(camera_id: u32, max_segments: usize) -> Self {
        Self {
            segments: VecDeque::with_capacity(max_segments),
            init_segment: None,
            sequence_number: 0,
            max_segments,
            camera_id,
        }
    }

    /// Add a new segment
    pub fn add_segment(&mut self, data: Vec<u8>, is_init: bool) -> u64 {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let segment = Segment {
            data,
            sequence: self.sequence_number,
            timestamp,
            duration_ms: 0, // TODO: Calculate from segment data
            is_init,
        };

        if is_init {
            info!("Camera {} received initialization segment ({} bytes)", 
                  self.camera_id, segment.data.len());
            self.init_segment = Some(segment);
        } else {
            debug!("Camera {} received media segment {} ({} bytes)", 
                   self.camera_id, self.sequence_number, segment.data.len());
            
            // Add to buffer
            self.segments.push_back(segment);
            
            // Remove old segments if buffer is full
            while self.segments.len() > self.max_segments {
                if let Some(removed) = self.segments.pop_front() {
                    debug!("Removed old segment {} from camera {}", 
                           removed.sequence, self.camera_id);
                }
            }
            
            self.sequence_number += 1;
        }

        self.sequence_number.saturating_sub(1)
    }

    /// Get the initialization segment
    pub fn get_init_segment(&self) -> Option<&Segment> {
        self.init_segment.as_ref()
    }

    /// Get a specific segment by sequence number
    pub fn get_segment(&self, sequence: u64) -> Option<&Segment> {
        self.segments.iter().find(|s| s.sequence == sequence)
    }

    /// Get the latest segment
    pub fn get_latest_segment(&self) -> Option<&Segment> {
        self.segments.back()
    }

    /// Get all segments starting from a sequence number
    pub fn get_segments_from(&self, start_sequence: u64) -> Vec<&Segment> {
        self.segments
            .iter()
            .filter(|s| s.sequence >= start_sequence)
            .collect()
    }

    /// Get current sequence number
    pub fn current_sequence(&self) -> u64 {
        self.sequence_number
    }

    /// Get segment count
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }
}

/// MSE client for a single camera
pub struct MseClient {
    camera_id: u32,
    stream_id: u32,
    width: u32,
    height: u32,
    is_registered: bool,
    segment_manager: Arc<Mutex<SegmentManager>>,
    segment_sender: broadcast::Sender<Segment>,
    socket_client: MseSocketClient,
}

impl MseClient {
    /// Create a new MSE client for a camera
    pub fn new(camera_id: u32, width: u32, height: u32) -> Result<Self> {
        let (segment_sender, _) = broadcast::channel(1000);
        
        let client = Self {
            camera_id,
            stream_id: 0, // Default stream ID
            width,
            height,
            is_registered: false,
            segment_manager: Arc::new(Mutex::new(SegmentManager::new(camera_id, 300))), // 5 minutes at 1fps
            segment_sender,
            socket_client: MseSocketClient::new(),
        };

        Ok(client)
    }

    /// Register the stream with the MSE plugin
    pub fn register_stream(&mut self) -> Result<()> {
        if self.is_registered {
            return Ok(());
        }

        // Use socket communication (new architecture)
        self.socket_client.register_stream(
            self.camera_id, 
            self.stream_id, 
            "h264", 
            self.width as i32, 
            self.height as i32
        ).map_err(|e| MseError::SocketError(e.to_string()))?;

        info!("Registered MSE stream for camera {} ({}x{}) via socket", 
              self.camera_id, self.width, self.height);
        self.is_registered = true;
        Ok(())
    }

    /// Unregister the stream
    pub fn unregister_stream(&mut self) {
        if !self.is_registered {
            return;
        }

        // Use socket communication (new architecture)
        if let Err(e) = self.socket_client.unregister_stream(self.camera_id, self.stream_id) {
            error!("Failed to unregister MSE stream for camera {}: {}", self.camera_id, e);
        } else {
            info!("Unregistered MSE stream for camera {}", self.camera_id);
        }

        self.is_registered = false;
    }

    /// Get next segment (blocking)
    pub fn get_segment(&self) -> Result<Vec<u8>, MseError> {
        // Use socket communication to try pop a segment (we'll poll until we get one)
        // Note: The new plugin architecture doesn't have a blocking pop, so we simulate it
        loop {
            match self.socket_client.try_pop_segment(self.camera_id) {
                Ok(Some(data)) => return Ok(data),
                Ok(None) => {
                    // Wait a bit and try again
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(e) => return Err(MseError::SocketError(e.to_string())),
            }
        }
    }

    /// Try to get segment (non-blocking)
    pub fn try_get_segment(&self) -> Result<Option<Vec<u8>>, MseError> {
        // Use socket communication to try pop a segment (non-blocking)
        self.socket_client.try_pop_segment(self.camera_id)
            .map_err(|e| MseError::SocketError(e.to_string()))
    }

    /// Get stream statistics
    pub fn get_stats(&self) -> MseStats {
        // Use socket communication to get stats
        let (buffer_size, total_segments, dropped_segments, bytes_received, frame_count) = match self.socket_client.get_buffer_stats(self.camera_id) {
            Ok(stats) => stats,
            Err(e) => {
                warn!("Failed to get stats via socket for camera {}: {}", self.camera_id, e);
                // Return default stats on error
                return MseStats {
                    camera_id: self.camera_id,
                    buffer_size: 0,
                    total_segments: 0,
                    dropped_segments: 0,
                    bytes_received: 0,
                    frame_count: 0,
                    active_clients: 0,
                    last_segment_time: 0,
                };
            }
        };

        // Use try_lock to avoid deadlock with background segment processing
        let last_segment_time = if let Ok(segment_manager) = self.segment_manager.try_lock() {
            segment_manager
                .get_latest_segment()
                .map(|s| s.timestamp)
                .unwrap_or(0)
        } else {
            // If we can't get the lock, just return 0 for last_segment_time
            warn!("Could not acquire segment manager lock for stats, using default value");
            0
        };

        MseStats {
            camera_id: self.camera_id,
            buffer_size,
            total_segments,
            dropped_segments,
            bytes_received,
            frame_count,
            active_clients: self.segment_sender.receiver_count(),
            last_segment_time,
        }
    }

    /// Subscribe to new segments
    pub fn subscribe(&self) -> broadcast::Receiver<Segment> {
        self.segment_sender.subscribe()
    }

    /// Get segment manager
    pub fn segment_manager(&self) -> Arc<Mutex<SegmentManager>> {
        self.segment_manager.clone()
    }

    /// Process segments in a background task
    pub fn start_segment_processing(&self) -> Result<()> {
        let camera_id = self.camera_id;
        let segment_manager = self.segment_manager.clone();
        let segment_sender = self.segment_sender.clone();
        let socket_client = MseSocketClient::new();

        tokio::spawn(async move {
            info!("Starting socket-based segment processing for camera {}", camera_id);
            
            let mut sequence = 0u64;
            let mut no_data_count = 0u64;
            
            loop {
                // Poll for segments from the socket-based plugin (non-blocking)
                match socket_client.try_pop_segment(camera_id) {
                    Ok(Some(segment_data)) => {
                        // We got a segment from the plugin
                        let is_init = sequence == 0; // First segment is initialization
                        
                        info!("Got segment from socket plugin for camera {}: {} bytes, seq {}", 
                               camera_id, segment_data.len(), sequence);
                        no_data_count = 0; // Reset counter
                        
                        // Add to segment manager
                        let _sequence_num = {
                            if let Ok(mut manager) = segment_manager.try_lock() {
                                manager.add_segment(segment_data.clone(), is_init)
                            } else {
                                warn!("Could not acquire segment manager lock for camera {}, skipping segment", camera_id);
                                continue;
                            }
                        };

                        // Create segment for broadcasting
                        let segment = Segment {
                            data: segment_data,
                            sequence,
                            timestamp: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis() as u64,
                            duration_ms: 0, // Will be calculated from segment data
                            is_init,
                        };

                        // Broadcast to subscribers
                        if let Err(_) = segment_sender.send(segment) {
                            debug!("No active subscribers for camera {}", camera_id);
                        }
                        
                        sequence += 1;
                    }
                    Ok(None) => {
                        // No segment available, wait a bit before polling again
                        no_data_count += 1;
                        
                        // Log periodically to show we're polling but not getting data
                        if no_data_count % 100 == 0 {
                            debug!("No data from socket plugin for camera {} after {} polls", 
                                   camera_id, no_data_count);
                            
                            // Check buffer status
                            if let Ok((buffer_size, total_segments, dropped_segments, bytes_received, frame_count)) = socket_client.get_buffer_stats(camera_id) {
                                debug!("Socket plugin stats for camera {}: buffer_size={}, total_segments={}, dropped_segments={}, bytes_received={}, frame_count={}", 
                                       camera_id, buffer_size, total_segments, dropped_segments, bytes_received, frame_count);
                            }
                        }
                        
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                    Err(e) => {
                        // Error occurred, wait a bit before polling again
                        no_data_count += 1;
                        
                        // Log periodically to show we're polling but getting errors
                        if no_data_count % 100 == 0 {
                            debug!("Error from socket plugin for camera {} after {} polls: {}", 
                                   camera_id, no_data_count, e);
                        }
                        
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                }
            }
        });

        Ok(())
    }

    /// Get camera ID
    pub fn camera_id(&self) -> u32 {
        self.camera_id
    }

    /// Get initialization segment directly from plugin
    pub fn get_init_segment_from_plugin(&self) -> Result<Option<Vec<u8>>, MseError> {
        // Use socket communication (new architecture)
        match self.socket_client.get_init_segment(self.camera_id) {
            Ok(data) => {
                if let Some(segment_data) = data {
                    info!("Got initialization segment via socket for camera {}: {} bytes", 
                          self.camera_id, segment_data.len());
                    Ok(Some(segment_data))
                } else {
                    debug!("No initialization segment available via socket for camera {}", self.camera_id);
                    Ok(None)
                }
            }
            Err(e) => {
                error!("Failed to get initialization segment for camera {}: {}", 
                       self.camera_id, e);
                Err(MseError::SocketError(e.to_string()))
            }
        }
    }

    /// Get latest segment directly from plugin
    pub fn get_latest_segment_from_plugin(&self) -> Result<Option<Vec<u8>>, MseError> {
        // Use socket communication (new architecture)
        match self.socket_client.get_latest_segment(self.camera_id) {
            Ok(data) => {
                if let Some(segment_data) = data {
                    info!("Got latest segment via socket for camera {}: {} bytes", 
                          self.camera_id, segment_data.len());
                    Ok(Some(segment_data))
                } else {
                    debug!("No latest segment available via socket for camera {}", self.camera_id);
                    Ok(None)
                }
            }
            Err(e) => {
                error!("Failed to get latest segment for camera {}: {}", 
                       self.camera_id, e);
                Err(MseError::SocketError(e.to_string()))
            }
        }
    }
}

impl Drop for MseClient {
    fn drop(&mut self) {
        self.unregister_stream();
    }
}

/// Manager for multiple MSE streams
pub struct MseStreamManager {
    clients: Arc<RwLock<HashMap<u32, Arc<MseClient>>>>,
}

impl MseStreamManager {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or create an MSE client for a camera
    pub async fn get_or_create_client(&self, camera_id: u32, width: u32, height: u32) -> Result<Arc<MseClient>> {
        // First, try to get existing client with a read lock
        {
            let clients = self.clients.read().await;
            if let Some(client) = clients.get(&camera_id) {
                debug!("Using existing MSE client for camera {}", camera_id);
                return Ok(client.clone());
            }
        }

        // Create new client outside of any locks to prevent deadlocks
        let mut client = MseClient::new(camera_id, width, height)?;
        
        // Register the stream
        client.register_stream()?;
        
        // Start processing (this spawns a background task, should not block)
        client.start_segment_processing()?;
        
        let client = Arc::new(client);
        
        // Now acquire write lock only for insertion
        {
            let mut clients = self.clients.write().await;
            
            // Double-check in case another task created it while we were working
            if let Some(existing) = clients.get(&camera_id) {
                debug!("MSE client was created by another task, using existing for camera {}", camera_id);
                return Ok(existing.clone());
            }
            
            clients.insert(camera_id, client.clone());
        }
        
        info!("Created and started MSE client for camera {}", camera_id);
        Ok(client)
    }

    /// Get an existing client
    pub async fn get_client(&self, camera_id: u32) -> Option<Arc<MseClient>> {
        let clients = self.clients.read().await;
        clients.get(&camera_id).cloned()
    }

    /// Remove a client
    pub async fn remove_client(&self, camera_id: u32) {
        let mut clients = self.clients.write().await;
        if let Some(_) = clients.remove(&camera_id) {
            info!("Removed MSE client for camera {}", camera_id);
        }
    }

    /// Get all active camera IDs
    pub async fn get_active_cameras(&self) -> Vec<u32> {
        let clients = self.clients.read().await;
        clients.keys().copied().collect()
    }

    /// Get statistics for all streams
    pub async fn get_all_stats(&self) -> HashMap<u32, MseStats> {
        let clients = self.clients.read().await;
        let mut stats = HashMap::new();
        
        for (camera_id, client) in clients.iter() {
            stats.insert(*camera_id, client.get_stats());
        }
        
        stats
    }
}

impl Default for MseStreamManager {
    fn default() -> Self {
        Self::new()
    }
}
