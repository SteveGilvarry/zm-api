use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use webrtc::peer_connection::RTCPeerConnection;

/// Unique session identifier
pub type SessionId = String;

/// WebRTC session state
#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    New,
    Connecting,
    Connected,
    Disconnected,
    Failed,
}

/// A WebRTC session for a single viewer
pub struct WebRtcSession {
    pub id: SessionId,
    pub monitor_id: u32,
    pub user_id: Option<String>,
    pub state: SessionState,
    pub peer_connection: Arc<RTCPeerConnection>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub connected_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Session statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionStats {
    pub id: SessionId,
    pub monitor_id: u32,
    pub state: String,
    pub duration_seconds: u64,
    pub bytes_sent: u64,
    pub packets_sent: u64,
}

/// Manages all active WebRTC sessions
pub struct SessionManager {
    sessions: DashMap<SessionId, Arc<RwLock<WebRtcSession>>>,
    max_sessions: usize,
}

impl SessionManager {
    /// Create a new session manager with a maximum session limit
    pub fn new(max_sessions: usize) -> Self {
        tracing::info!("Initializing SessionManager with max_sessions={}", max_sessions);
        Self {
            sessions: DashMap::new(),
            max_sessions,
        }
    }

    /// Create a new WebRTC session
    pub fn create_session(
        &self,
        monitor_id: u32,
        user_id: Option<String>,
        pc: Arc<RTCPeerConnection>,
    ) -> Result<SessionId, SessionError> {
        // Check if we've reached the maximum session limit
        if self.sessions.len() >= self.max_sessions {
            tracing::warn!(
                "Maximum sessions reached: {}/{}",
                self.sessions.len(),
                self.max_sessions
            );
            return Err(SessionError::MaxSessionsReached(self.max_sessions));
        }

        // Generate a unique session ID
        let session_id = uuid::Uuid::new_v4().to_string();

        // Check if session already exists (extremely unlikely with UUIDs, but defensive)
        if self.sessions.contains_key(&session_id) {
            tracing::error!("Session ID collision detected: {}", session_id);
            return Err(SessionError::AlreadyExists(session_id));
        }

        let session = WebRtcSession {
            id: session_id.clone(),
            monitor_id,
            user_id: user_id.clone(),
            state: SessionState::New,
            peer_connection: pc,
            created_at: chrono::Utc::now(),
            connected_at: None,
        };

        self.sessions
            .insert(session_id.clone(), Arc::new(RwLock::new(session)));

        tracing::info!(
            "Created session {} for monitor {} (user: {:?})",
            session_id,
            monitor_id,
            user_id
        );

        Ok(session_id)
    }

    /// Get a session by ID
    pub fn get_session(&self, id: &SessionId) -> Option<Arc<RwLock<WebRtcSession>>> {
        self.sessions.get(id).map(|entry| entry.value().clone())
    }

    /// Update the state of a session
    pub fn update_state(&self, id: &SessionId, state: SessionState) -> Result<(), SessionError> {
        if let Some(session_lock) = self.sessions.get(id) {
            let session_arc = session_lock.value().clone();
            drop(session_lock); // Release the DashMap lock

            // Spawn a task to update the session state
            tokio::spawn(async move {
                let mut session = session_arc.write().await;
                let old_state = session.state.clone();
                session.state = state.clone();

                // Update connected_at timestamp when transitioning to Connected
                if state == SessionState::Connected && session.connected_at.is_none() {
                    session.connected_at = Some(chrono::Utc::now());
                }

                tracing::debug!(
                    "Session {} state changed: {:?} -> {:?}",
                    session.id,
                    old_state,
                    state
                );
            });

            Ok(())
        } else {
            tracing::warn!("Attempted to update non-existent session: {}", id);
            Err(SessionError::NotFound(id.clone()))
        }
    }

    /// Remove a session
    pub fn remove_session(&self, id: &SessionId) -> Option<Arc<RwLock<WebRtcSession>>> {
        match self.sessions.remove(id) {
            Some((session_id, session_arc)) => {
                tracing::info!("Removed session {}", session_id);
                Some(session_arc)
            }
            None => {
                tracing::debug!("Attempted to remove non-existent session: {}", id);
                None
            }
        }
    }

    /// Get all session IDs for a specific monitor
    pub fn get_sessions_for_monitor(&self, monitor_id: u32) -> Vec<SessionId> {
        let mut session_ids = Vec::new();

        for entry in self.sessions.iter() {
            let session_arc = entry.value().clone();
            // We need to block on the async read since this method is sync
            // In a real scenario, you might want to make this async
            if let Ok(session) = session_arc.try_read() {
                if session.monitor_id == monitor_id {
                    session_ids.push(entry.key().clone());
                }
            };  // Add semicolon to drop the guard immediately
        }

        tracing::debug!(
            "Found {} sessions for monitor {}",
            session_ids.len(),
            monitor_id
        );

        session_ids
    }

    /// Get the count of active sessions
    pub fn active_session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Get statistics for a session
    pub async fn get_session_stats(&self, id: &SessionId) -> Option<SessionStats> {
        let session_lock = self.sessions.get(id)?;
        let session_arc = session_lock.value().clone();
        drop(session_lock); // Release the DashMap lock

        let session = session_arc.read().await;

        let duration_seconds = if let Some(connected_at) = session.connected_at {
            (chrono::Utc::now() - connected_at).num_seconds() as u64
        } else {
            (chrono::Utc::now() - session.created_at).num_seconds() as u64
        };

        // Get stats from the peer connection
        let stats = session.peer_connection.get_stats().await;

        // Aggregate bytes and packets sent
        let mut bytes_sent = 0u64;
        let mut packets_sent = 0u64;

        for report in stats.reports.values() {
            // Extract outbound RTP stream stats
            use webrtc::stats::StatsReportType;
            if let StatsReportType::OutboundRTP(outbound_rtp) = report {
                bytes_sent += outbound_rtp.bytes_sent;
                packets_sent += outbound_rtp.packets_sent;
            }
        }

        Some(SessionStats {
            id: session.id.clone(),
            monitor_id: session.monitor_id,
            state: format!("{:?}", session.state),
            duration_seconds,
            bytes_sent,
            packets_sent,
        })
    }

    /// Clean up stale sessions that have been inactive for longer than max_age
    pub async fn cleanup_stale_sessions(&self, max_age: std::time::Duration) -> usize {
        let now = chrono::Utc::now();
        let mut stale_sessions = Vec::new();

        // Find stale sessions
        for entry in self.sessions.iter() {
            let session_arc = entry.value().clone();
            let session = session_arc.read().await;

            let age = if let Some(connected_at) = session.connected_at {
                now - connected_at
            } else {
                now - session.created_at
            };

            // Convert age to std::time::Duration for comparison
            let age_duration = age.to_std().unwrap_or(std::time::Duration::from_secs(0));

            // Mark disconnected or failed sessions that are old enough
            if (session.state == SessionState::Disconnected
                || session.state == SessionState::Failed)
                && age_duration > max_age
            {
                stale_sessions.push(session.id.clone());
            }
        }

        // Remove stale sessions
        let count = stale_sessions.len();
        for session_id in stale_sessions {
            self.remove_session(&session_id);
        }

        if count > 0 {
            tracing::info!("Cleaned up {} stale sessions", count);
        }

        count
    }
}

/// Errors that can occur during session management
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Maximum sessions reached ({0})")]
    MaxSessionsReached(usize),
    #[error("Session not found: {0}")]
    NotFound(SessionId),
    #[error("Session already exists: {0}")]
    AlreadyExists(SessionId),
}

#[cfg(test)]
mod tests {
    use super::*;
    use webrtc::api::APIBuilder;
    use webrtc::api::media_engine::MediaEngine;
    use webrtc::peer_connection::configuration::RTCConfiguration;

    async fn create_test_peer_connection() -> Arc<RTCPeerConnection> {
        let mut media_engine = MediaEngine::default();
        media_engine.register_default_codecs().unwrap();

        let api = APIBuilder::new()
            .with_media_engine(media_engine)
            .build();

        let config = RTCConfiguration::default();
        Arc::new(api.new_peer_connection(config).await.unwrap())
    }

    #[tokio::test]
    async fn test_create_session() {
        let manager = SessionManager::new(10);
        let pc = create_test_peer_connection().await;

        let session_id = manager
            .create_session(1, Some("user123".to_string()), pc)
            .unwrap();

        assert!(!session_id.is_empty());
        assert_eq!(manager.active_session_count(), 1);
    }

    #[tokio::test]
    async fn test_max_sessions() {
        let manager = SessionManager::new(2);

        for _ in 0..2 {
            let pc = create_test_peer_connection().await;
            manager.create_session(1, None, pc).unwrap();
        }

        let pc = create_test_peer_connection().await;
        let result = manager.create_session(1, None, pc);

        assert!(matches!(result, Err(SessionError::MaxSessionsReached(2))));
    }

    #[tokio::test]
    async fn test_get_session() {
        let manager = SessionManager::new(10);
        let pc = create_test_peer_connection().await;
        let session_id = manager.create_session(1, None, pc).unwrap();

        let session = manager.get_session(&session_id);
        assert!(session.is_some());

        let session = session.unwrap();
        let session_data = session.read().await;
        assert_eq!(session_data.id, session_id);
        assert_eq!(session_data.monitor_id, 1);
        assert_eq!(session_data.state, SessionState::New);
    }

    #[tokio::test]
    async fn test_update_state() {
        let manager = SessionManager::new(10);
        let pc = create_test_peer_connection().await;
        let session_id = manager.create_session(1, None, pc).unwrap();

        manager
            .update_state(&session_id, SessionState::Connecting)
            .unwrap();

        // Give the async task time to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let session = manager.get_session(&session_id).unwrap();
        let session_data = session.read().await;
        assert_eq!(session_data.state, SessionState::Connecting);
    }

    #[tokio::test]
    async fn test_remove_session() {
        let manager = SessionManager::new(10);
        let pc = create_test_peer_connection().await;
        let session_id = manager.create_session(1, None, pc).unwrap();

        assert_eq!(manager.active_session_count(), 1);

        let removed = manager.remove_session(&session_id);
        assert!(removed.is_some());
        assert_eq!(manager.active_session_count(), 0);

        let removed_again = manager.remove_session(&session_id);
        assert!(removed_again.is_none());
    }

    #[tokio::test]
    async fn test_get_sessions_for_monitor() {
        let manager = SessionManager::new(10);

        for monitor_id in 1..=3 {
            for _ in 0..2 {
                let pc = create_test_peer_connection().await;
                manager.create_session(monitor_id, None, pc).unwrap();
            }
        }

        let monitor1_sessions = manager.get_sessions_for_monitor(1);
        assert_eq!(monitor1_sessions.len(), 2);

        let monitor2_sessions = manager.get_sessions_for_monitor(2);
        assert_eq!(monitor2_sessions.len(), 2);

        let monitor99_sessions = manager.get_sessions_for_monitor(99);
        assert_eq!(monitor99_sessions.len(), 0);
    }

    #[tokio::test]
    async fn test_cleanup_stale_sessions() {
        let manager = SessionManager::new(10);
        let pc = create_test_peer_connection().await;
        let session_id = manager.create_session(1, None, pc).unwrap();

        // Mark as disconnected
        manager
            .update_state(&session_id, SessionState::Disconnected)
            .unwrap();

        // Give the async task time to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Clean up sessions older than 0 seconds (should remove the disconnected session)
        let count = manager
            .cleanup_stale_sessions(std::time::Duration::from_secs(0))
            .await;

        assert_eq!(count, 1);
        assert_eq!(manager.active_session_count(), 0);
    }
}
