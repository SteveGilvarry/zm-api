//! MSE (Media Source Extensions) live streaming
//!
//! Provides lower-latency streaming via WebSocket using fMP4 fragments
//! that are fed directly to the browser's Media Source Extensions API.

use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use tokio::sync::{broadcast, RwLock};
use tracing::info;
use uuid::Uuid;

use crate::streaming::hls::segmenter::{HlsSegmenter, InitSegment};
use crate::streaming::source::FifoPacket;

/// MSE session state
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MseSessionState {
    /// Session created, waiting for WebSocket connection
    Pending,
    /// WebSocket connected, streaming
    Active,
    /// Session closed
    Closed,
}

/// MSE segment for WebSocket transmission
#[derive(Debug, Clone)]
pub struct MseSegment {
    /// Segment sequence number
    pub sequence: u64,
    /// Segment data (fMP4 fragment)
    pub data: Vec<u8>,
    /// Duration in milliseconds
    pub duration_ms: u32,
    /// Whether this starts with a keyframe
    pub is_keyframe: bool,
    /// Timestamp in microseconds
    pub timestamp_us: u64,
}

/// Statistics for an MSE session
#[derive(Debug, Clone, serde::Serialize)]
pub struct MseSessionStats {
    pub session_id: String,
    pub monitor_id: u32,
    pub state: MseSessionState,
    pub segments_sent: u64,
    pub bytes_sent: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// MSE session for a single viewer
pub struct MseSession {
    pub id: Uuid,
    pub monitor_id: u32,
    pub state: MseSessionState,
    /// Channel for sending segments to WebSocket handler
    segment_tx: broadcast::Sender<MseSegment>,
    /// Init segment (cached after generation)
    init_segment: RwLock<Option<InitSegment>>,
    /// Segmenter for generating fMP4 fragments
    segmenter: RwLock<HlsSegmenter>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    segments_sent: RwLock<u64>,
    bytes_sent: RwLock<u64>,
}

impl MseSession {
    /// Create a new MSE session
    pub fn new(monitor_id: u32, segment_duration_ms: u32) -> Self {
        let (segment_tx, _) = broadcast::channel(32);
        let target_duration = Duration::from_millis(segment_duration_ms as u64);

        Self {
            id: Uuid::new_v4(),
            monitor_id,
            state: MseSessionState::Pending,
            segment_tx,
            init_segment: RwLock::new(None),
            segmenter: RwLock::new(HlsSegmenter::new(monitor_id, target_duration)),
            created_at: chrono::Utc::now(),
            segments_sent: RwLock::new(0),
            bytes_sent: RwLock::new(0),
        }
    }

    /// Subscribe to receive segments
    pub fn subscribe(&self) -> broadcast::Receiver<MseSegment> {
        self.segment_tx.subscribe()
    }

    /// Get the init segment
    pub async fn get_init_segment(&self) -> Option<InitSegment> {
        self.init_segment.read().await.clone()
    }

    /// Process a packet and potentially produce segments
    pub async fn process_packet(&self, packet: &FifoPacket) {
        let mut segmenter = self.segmenter.write().await;

        // Set codec if not already set
        if segmenter.sequence() == 0 {
            segmenter.set_codec(packet.codec);
        }

        let timestamp_us = match u64::try_from(packet.timestamp_us) {
            Ok(ts) => ts,
            Err(_) => return,
        };

        // Process NAL
        if let Some(fmp4_segment) =
            segmenter.process_nal(&packet.data, timestamp_us, packet.is_keyframe)
        {
            let mse_segment = MseSegment {
                sequence: fmp4_segment.sequence,
                data: fmp4_segment.data,
                duration_ms: fmp4_segment.duration.as_millis() as u32,
                is_keyframe: fmp4_segment.is_keyframe,
                timestamp_us: fmp4_segment.timestamp * 1_000_000 / 90000, // Convert from 90kHz to us
            };

            // Update stats
            {
                let mut sent = self.segments_sent.write().await;
                *sent += 1;
            }
            {
                let mut bytes = self.bytes_sent.write().await;
                *bytes += mse_segment.data.len() as u64;
            }

            // Send to subscribers (ignore errors if no receivers)
            let _ = self.segment_tx.send(mse_segment);
        }

        // Generate init segment if not already done
        let mut init_guard = self.init_segment.write().await;
        if init_guard.is_none() {
            if let Some(init) = segmenter.generate_init_segment() {
                *init_guard = Some(init);
            }
        }
    }

    /// Get session statistics
    pub async fn stats(&self) -> MseSessionStats {
        MseSessionStats {
            session_id: self.id.to_string(),
            monitor_id: self.monitor_id,
            state: self.state,
            segments_sent: *self.segments_sent.read().await,
            bytes_sent: *self.bytes_sent.read().await,
            created_at: self.created_at,
        }
    }
}

/// MSE live streaming errors
#[derive(Debug, thiserror::Error)]
pub enum MseLiveError {
    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Maximum sessions reached")]
    MaxSessionsReached,

    #[error("Init segment not ready")]
    InitNotReady,
}

/// Configuration for MSE live streaming
#[derive(Debug, Clone)]
pub struct MseLiveConfig {
    /// Target segment duration in milliseconds (smaller = lower latency)
    pub segment_duration_ms: u32,
    /// Maximum number of active sessions
    pub max_sessions: usize,
    /// Channel buffer size for segments
    pub buffer_size: usize,
}

impl Default for MseLiveConfig {
    fn default() -> Self {
        Self {
            segment_duration_ms: 500, // 500ms segments for lower latency
            max_sessions: 100,
            buffer_size: 32,
        }
    }
}

/// Manager for MSE live streaming sessions
pub struct MseLiveManager {
    config: MseLiveConfig,
    sessions: DashMap<String, Arc<MseSession>>,
    /// Map from monitor_id to list of session IDs
    monitor_sessions: DashMap<u32, Vec<String>>,
}

impl MseLiveManager {
    /// Create a new MSE live manager
    pub fn new(config: MseLiveConfig) -> Self {
        Self {
            config,
            sessions: DashMap::new(),
            monitor_sessions: DashMap::new(),
        }
    }

    /// Create a new MSE session for a monitor
    pub fn create_session(&self, monitor_id: u32) -> Result<Arc<MseSession>, MseLiveError> {
        // Check session limit
        if self.sessions.len() >= self.config.max_sessions {
            return Err(MseLiveError::MaxSessionsReached);
        }

        let session = Arc::new(MseSession::new(monitor_id, self.config.segment_duration_ms));
        let session_id = session.id.to_string();

        self.sessions.insert(session_id.clone(), session.clone());

        // Track by monitor
        self.monitor_sessions
            .entry(monitor_id)
            .or_default()
            .push(session_id.clone());

        info!(
            "Created MSE session {} for monitor {}",
            session_id, monitor_id
        );

        Ok(session)
    }

    /// Get session by ID
    pub fn get_session(&self, session_id: &str) -> Option<Arc<MseSession>> {
        self.sessions.get(session_id).map(|s| s.clone())
    }

    /// Remove a session
    pub fn remove_session(&self, session_id: &str) -> Result<(), MseLiveError> {
        let session = self
            .sessions
            .remove(session_id)
            .ok_or_else(|| MseLiveError::SessionNotFound(session_id.to_string()))?;

        // Remove from monitor sessions
        if let Some(mut sessions) = self.monitor_sessions.get_mut(&session.1.monitor_id) {
            sessions.retain(|id| id != session_id);
        }

        info!("Removed MSE session {}", session_id);
        Ok(())
    }

    /// Get all sessions for a monitor
    pub fn get_monitor_sessions(&self, monitor_id: u32) -> Vec<Arc<MseSession>> {
        self.monitor_sessions
            .get(&monitor_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.sessions.get(id).map(|s| s.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Process a packet for all sessions of a monitor
    pub async fn process_packet(&self, packet: &FifoPacket) {
        let sessions = self.get_monitor_sessions(packet.monitor_id);
        for session in sessions {
            session.process_packet(packet).await;
        }
    }

    /// Get session count
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// List all sessions
    pub fn list_sessions(&self) -> Vec<String> {
        self.sessions.iter().map(|r| r.key().clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mse_live_config_default() {
        let config = MseLiveConfig::default();
        assert_eq!(config.segment_duration_ms, 500);
        assert_eq!(config.max_sessions, 100);
    }

    #[test]
    fn test_session_state_serialize() {
        let state = MseSessionState::Active;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"active\"");
    }

    #[test]
    fn test_manager_creation() {
        let config = MseLiveConfig::default();
        let manager = MseLiveManager::new(config);
        assert_eq!(manager.session_count(), 0);
    }

    #[test]
    fn test_create_session() {
        let config = MseLiveConfig::default();
        let manager = MseLiveManager::new(config);

        let session = manager.create_session(1).unwrap();
        assert_eq!(session.monitor_id, 1);
        assert_eq!(manager.session_count(), 1);
    }
}
