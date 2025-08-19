/// Socket-based MSE client for the new plugin architecture
/// Communicates with MSE plugin TCP server on 127.0.0.1:9051

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use anyhow::Result;
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json;
use tracing::{debug, info, warn};

/// MSE socket client for communicating with the plugin TCP server
pub struct MseSocketClient {
    server_address: String,
    timeout: Duration,
}

/// JSON command types for MSE plugin communication
#[derive(Debug, Serialize)]
#[serde(tag = "command")]
pub enum MseCommand {
    #[serde(rename = "get_init_segment")]
    GetInitSegment {
        camera_id: u32,
    },
    #[serde(rename = "get_latest_segment")]
    GetLatestSegment {
        camera_id: u32,
    },
    #[serde(rename = "pop_segment")]
    PopSegment {
        camera_id: u32,
    },
    #[serde(rename = "try_pop_segment")]
    TryPopSegment {
        camera_id: u32,
    },
    #[serde(rename = "get_buffer_stats")]
    GetBufferStats {
        camera_id: u32,
    },
    #[serde(rename = "get_buffer_size")]
    GetBufferSize {
        camera_id: u32,
    },
    #[serde(rename = "register_stream")]
    RegisterStream {
        camera_id: u32,
        stream_id: u32,
        codec: String,
        width: i32,
        height: i32,
    },
    #[serde(rename = "unregister_stream")]
    UnregisterStream {
        camera_id: u32,
        stream_id: u32,
    },
}

/// JSON response from MSE plugin
#[derive(Debug, Deserialize)]
#[serde(tag = "status")]
pub enum MseResponse {
    #[serde(rename = "success")]
    Success {
        #[serde(default)]
        data: Option<String>, // Base64 encoded binary data
        #[serde(default)]
        size: Option<usize>,
        #[serde(default)]
        buffer_size: Option<usize>,
        #[serde(default)]
        total_segments: Option<u64>,
        #[serde(default)]
        dropped_segments: Option<u64>,
        #[serde(default)]
        bytes_received: Option<u64>,
        #[serde(default)]
        frame_count: Option<u64>,
    },
    #[serde(rename = "error")]
    Error {
        message: String,
        #[serde(default)]
        code: Option<i32>,
    },
}

impl MseSocketClient {
    /// Create a new MSE socket client
    pub fn new() -> Self {
        Self {
            server_address: "127.0.0.1:9051".to_string(),
            timeout: Duration::from_secs(5),
        }
    }

    /// Create a new MSE socket client with custom address
    pub fn with_address(address: String) -> Self {
        Self {
            server_address: address,
            timeout: Duration::from_secs(5),
        }
    }

    /// Connect to the MSE plugin server and send a command
    fn send_command(&self, command: &MseCommand) -> Result<MseResponse> {
        debug!("Connecting to MSE plugin at {}", self.server_address);
        
        let mut stream = TcpStream::connect(&self.server_address)
            .map_err(|e| anyhow::anyhow!("Failed to connect to MSE plugin: {}", e))?;
        
        stream.set_read_timeout(Some(self.timeout))?;
        stream.set_write_timeout(Some(self.timeout))?;

        // Serialize and send command
        let command_json = serde_json::to_string(command)?;
        debug!("Sending MSE command: {}", command_json);
        
        stream.write_all(command_json.as_bytes())?;
        stream.write_all(b"\n")?; // Add newline delimiter
        stream.flush()?;

        // Read response
        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer)?;
        
        let response_str = String::from_utf8(buffer)
            .map_err(|e| anyhow::anyhow!("Invalid UTF-8 response: {}", e))?;
        
        debug!("Received MSE response: {}", response_str);
        
        let response: MseResponse = serde_json::from_str(&response_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse MSE response: {}", e))?;
        
        Ok(response)
    }

    /// Register a stream with the MSE plugin
    pub fn register_stream(&self, camera_id: u32, stream_id: u32, codec: &str, width: i32, height: i32) -> Result<()> {
        let command = MseCommand::RegisterStream {
            camera_id,
            stream_id,
            codec: codec.to_string(),
            width,
            height,
        };
        
        match self.send_command(&command)? {
            MseResponse::Success { .. } => {
                info!("Successfully registered stream for camera {}", camera_id);
                Ok(())
            }
            MseResponse::Error { message, code } => {
                Err(anyhow::anyhow!("Failed to register stream: {} (code: {:?})", message, code))
            }
        }
    }

    /// Unregister a stream
    pub fn unregister_stream(&self, camera_id: u32, stream_id: u32) -> Result<()> {
        let command = MseCommand::UnregisterStream {
            camera_id,
            stream_id,
        };
        
        match self.send_command(&command)? {
            MseResponse::Success { .. } => {
                info!("Successfully unregistered stream for camera {}", camera_id);
                Ok(())
            }
            MseResponse::Error { message, code } => {
                Err(anyhow::anyhow!("Failed to unregister stream: {} (code: {:?})", message, code))
            }
        }
    }

    /// Get initialization segment
    pub fn get_init_segment(&self, camera_id: u32) -> Result<Option<Vec<u8>>> {
        let command = MseCommand::GetInitSegment { camera_id };
        
        match self.send_command(&command)? {
            MseResponse::Success { data: Some(base64_data), size: _, .. } => {
                let decoded = base64::engine::general_purpose::STANDARD.decode(base64_data)
                    .map_err(|e| anyhow::anyhow!("Failed to decode base64 data: {}", e))?;
                
                info!("Got initialization segment for camera {}: {} bytes", camera_id, decoded.len());
                Ok(Some(decoded))
            }
            MseResponse::Success { data: None, .. } => {
                debug!("No initialization segment available for camera {}", camera_id);
                Ok(None)
            }
            MseResponse::Error { message, code } => {
                warn!("Failed to get init segment: {} (code: {:?})", message, code);
                Ok(None)
            }
        }
    }

    /// Get latest segment
    pub fn get_latest_segment(&self, camera_id: u32) -> Result<Option<Vec<u8>>> {
        let command = MseCommand::GetLatestSegment { camera_id };
        
        match self.send_command(&command)? {
            MseResponse::Success { data: Some(base64_data), size: _, .. } => {
                let decoded = base64::engine::general_purpose::STANDARD.decode(base64_data)
                    .map_err(|e| anyhow::anyhow!("Failed to decode base64 data: {}", e))?;
                
                info!("Got latest segment for camera {}: {} bytes", camera_id, decoded.len());
                Ok(Some(decoded))
            }
            MseResponse::Success { data: None, .. } => {
                debug!("No latest segment available for camera {}", camera_id);
                Ok(None)
            }
            MseResponse::Error { message, code } => {
                warn!("Failed to get latest segment: {} (code: {:?})", message, code);
                Ok(None)
            }
        }
    }

    /// Try to pop a segment (non-blocking)
    pub fn try_pop_segment(&self, camera_id: u32) -> Result<Option<Vec<u8>>> {
        let command = MseCommand::TryPopSegment { camera_id };
        
        match self.send_command(&command)? {
            MseResponse::Success { data: Some(base64_data), .. } => {
                let decoded = base64::engine::general_purpose::STANDARD.decode(base64_data)
                    .map_err(|e| anyhow::anyhow!("Failed to decode base64 data: {}", e))?;
                
                debug!("Popped segment for camera {}: {} bytes", camera_id, decoded.len());
                Ok(Some(decoded))
            }
            MseResponse::Success { data: None, .. } => {
                Ok(None)
            }
            MseResponse::Error { message, code } => {
                warn!("Failed to pop segment: {} (code: {:?})", message, code);
                Ok(None)
            }
        }
    }

    /// Get buffer statistics
    pub fn get_buffer_stats(&self, camera_id: u32) -> Result<(usize, u64, u64, u64, u64)> {
        let command = MseCommand::GetBufferStats { camera_id };
        
        match self.send_command(&command)? {
            MseResponse::Success { 
                buffer_size, 
                total_segments, 
                dropped_segments, 
                bytes_received, 
                frame_count, 
                .. 
            } => {
                Ok((
                    buffer_size.unwrap_or(0),
                    total_segments.unwrap_or(0),
                    dropped_segments.unwrap_or(0),
                    bytes_received.unwrap_or(0),
                    frame_count.unwrap_or(0),
                ))
            }
            MseResponse::Error { message, code } => {
                Err(anyhow::anyhow!("Failed to get buffer stats: {} (code: {:?})", message, code))
            }
        }
    }

    /// Get buffer size
    pub fn get_buffer_size(&self, camera_id: u32) -> Result<usize> {
        let command = MseCommand::GetBufferSize { camera_id };
        
        match self.send_command(&command)? {
            MseResponse::Success { buffer_size: Some(size), .. } => Ok(size),
            MseResponse::Success { .. } => Ok(0),
            MseResponse::Error { message, code } => {
                Err(anyhow::anyhow!("Failed to get buffer size: {} (code: {:?})", message, code))
            }
        }
    }
}

impl Default for MseSocketClient {
    fn default() -> Self {
        Self::new()
    }
}
