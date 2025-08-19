use std::sync::Arc;
use anyhow::{Result, anyhow};
use tracing::{info, error, warn};
use sea_orm::DatabaseConnection;

use crate::webrtc_ffi::{init_webrtc_service, list_camera_streams, CameraStream, WebRTCSession};
use crate::webrtc_signaling::{WebRTCSignalingService, StreamInfo};
use crate::repo::monitors::MonitorRepository;

/// WebRTC service that integrates with the centralized zm-next WebRTC service
pub struct WebRTCService {
    signaling_service: Arc<WebRTCSignalingService>,
    monitor_repo: Arc<MonitorRepository>,
    instance_id: String,
}

impl WebRTCService {
    /// Create a new WebRTC service
    pub fn new(db: Arc<DatabaseConnection>) -> Result<Self> {
        let instance_id = "zm_api_instance".to_string();
        
        // Initialize the WebRTC service for camera 0 (default camera)
        if let Err(e) = init_webrtc_service(0) {
            warn!("Failed to initialize WebRTC service (may already be running): {}", e);
        }
        
        let signaling_service = Arc::new(WebRTCSignalingService::new()
            .map_err(|e| anyhow!("Failed to create WebRTC signaling service: {}", e))?);
        let monitor_repo = Arc::new(MonitorRepository::new(db));

        Ok(Self {
            signaling_service,
            monitor_repo,
            instance_id,
        })
    }

    /// Get the signaling service
    pub fn signaling_service(&self) -> Arc<WebRTCSignalingService> {
        self.signaling_service.clone()
    }

    /// Discover available camera streams from the centralized service
    pub async fn discover_camera_streams(&self) -> Result<Vec<CameraStream>> {
        match list_camera_streams() {
            Ok(streams) => {
                info!("Discovered {} camera streams", streams.len());
                for stream in &streams {
                    info!("Stream: {} - {} ({})", stream.stream_id, stream.camera_name, 
                         if stream.is_active { "active" } else { "inactive" });
                }
                Ok(streams)
            }
            Err(e) => {
                error!("Failed to discover camera streams: {}", e);
                Ok(Vec::new()) // Return empty list instead of failing
            }
        }
    }

    /// Create a new WebRTC client session
    pub fn create_client_session(&self) -> Result<WebRTCSession> {
        // Use camera 0 as default for now
        WebRTCSession::new(0)
    }

    /// Update monitors from database and sync with discovered streams
    pub async fn refresh_monitors(&self) -> Result<()> {
        info!("Refreshing monitors and syncing with camera streams");
        
        // Discover available streams
        let camera_streams = self.discover_camera_streams().await?;
        
        // Get monitors from database
        let monitors = self.monitor_repo.find_all().await
            .map_err(|e| anyhow!("Failed to fetch monitors: {}", e))?;
        
        info!("Found {} monitors in database, {} camera streams available", 
              monitors.len(), camera_streams.len());
        
        // TODO: Implement logic to match monitors with camera streams
        // For now, just log the information
        for monitor in &monitors {
            if monitor.enabled == 1 {
                info!("Monitor {} ({}) is enabled", monitor.id, monitor.name);
            }
        }
        
        Ok(())
    }

    /// Check if a monitor exists and is enabled
    pub async fn is_monitor_available(&self, monitor_id: i32) -> Result<bool> {
        let monitor = self.monitor_repo.find_by_id(monitor_id).await
            .map_err(|e| anyhow!("Failed to check monitor: {}", e))?;

        Ok(monitor.map(|m| m.enabled == 1).unwrap_or(false))
    }

    /// Get monitor details as stream info
    pub async fn get_monitor_info(&self, monitor_id: i32) -> Result<Option<StreamInfo>> {
        let monitor = self.monitor_repo.find_by_id(monitor_id).await
            .map_err(|e| anyhow!("Failed to get monitor: {}", e))?;

        Ok(monitor.map(|m| StreamInfo {
            id: format!("stream_{}", m.id),
            name: m.name,
            description: Some(format!("Monitor {} stream", m.id)),
            active: m.enabled == 1,
            client_count: 0, // This would be updated by the signaling service
            resolution: format!("{}x{}", m.width.unwrap_or(1920), m.height.unwrap_or(1080)),
        }))
    }

    /// Get available camera streams as StreamInfo objects
    pub async fn get_available_streams(&self) -> Result<Vec<StreamInfo>> {
        let camera_streams = self.discover_camera_streams().await?;
        
        let stream_infos = camera_streams.into_iter().map(|stream| StreamInfo {
            id: stream.stream_id,
            name: stream.camera_name,
            description: Some(format!("Camera stream - {}", stream.resolution)),
            active: stream.is_active,
            client_count: 0,
            resolution: stream.resolution,
        }).collect();
        
        Ok(stream_infos)
    }

    /// Get WebRTC statistics
    pub async fn get_stats(&self) -> WebRTCStats {
        let signaling_stats = self.signaling_service.get_stats().await;
        let camera_streams = self.discover_camera_streams().await.unwrap_or_default();
        
        WebRTCStats {
            connected_clients: signaling_stats.get("total_clients")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as i32,
            active_monitors: camera_streams.iter().filter(|s| s.is_active).count() as i32,
            total_streams: camera_streams.len() as i32,
            service_status: "connected".to_string(),
        }
    }

    /// Start periodic monitor refresh task
    pub async fn start_monitor_refresh_task(&self) {
        let service = Arc::new(self.clone());
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = service.refresh_monitors().await {
                    error!("Failed to refresh monitors: {}", e);
                } else {
                    info!("WebRTC monitor refresh completed");
                }
            }
        });
        
        info!("Started WebRTC monitor refresh task (30s interval)");
    }
}

impl Clone for WebRTCService {
    fn clone(&self) -> Self {
        Self {
            signaling_service: Arc::clone(&self.signaling_service),
            monitor_repo: Arc::clone(&self.monitor_repo),
            instance_id: self.instance_id.clone(),
        }
    }
}

/// WebRTC service statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct WebRTCStats {
    pub connected_clients: i32,
    pub active_monitors: i32,
    pub total_streams: i32,
    pub service_status: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webrtc_stats_serialization() {
        let stats = WebRTCStats {
            connected_clients: 5,
            active_monitors: 3,
            total_streams: 5,
            service_status: "connected".to_string(),
        };
        
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("connected_clients"));
        assert!(json.contains("active_monitors"));
        assert!(json.contains("total_streams"));
        assert!(json.contains("service_status"));
    }
}
