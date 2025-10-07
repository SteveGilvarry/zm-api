use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use anyhow::{Result, anyhow};
use libloading::{Library, Symbol};
use once_cell::sync::OnceCell;
use uuid::Uuid;

/// WebRTC session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionState {
    New = 0,
    Connecting = 1,
    Connected = 2,
    Disconnected = 3,
    Failed = 4,
    Closed = 5,
}

impl From<i32> for SessionState {
    fn from(state: i32) -> Self {
        match state {
            0 => SessionState::New,
            1 => SessionState::Connecting,
            2 => SessionState::Connected,
            3 => SessionState::Disconnected,
            4 => SessionState::Failed,
            5 => SessionState::Closed,
            _ => SessionState::Failed,
        }
    }
}

/// WebRTC ICE candidate structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceCandidate {
    pub candidate: String,
    pub sdp_mid: Option<String>,
    pub sdp_mline_index: Option<i32>,
}

/// WebRTC session description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDescription {
    pub sdp_type: String, // "offer" or "answer"
    pub sdp: String,
}

/// Camera stream information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CameraStream {
    pub stream_id: String,
    pub camera_name: String,
    pub resolution: String,
    pub is_active: bool,
}

/// WebRTC service statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRTCStats {
    pub total_frames: u64,
    pub total_bytes: u64,
    pub clients_connected: u64,
    pub clients_disconnected: u64,
}

/// Function type definitions for dynamic loading
type InitServiceFn = unsafe extern "C" fn(u32) -> c_int;
type ShutdownServiceFn = unsafe extern "C" fn(u32) -> c_int;
type ListCameraStreamsFn = unsafe extern "C" fn() -> *mut c_char;
type RegisterStreamFn = unsafe extern "C" fn(u32) -> c_int;
type UnregisterStreamFn = unsafe extern "C" fn(u32) -> c_int;
type CreateClientFn = unsafe extern "C" fn(*const c_char, u32) -> *mut c_void;
type SetOfferFn = unsafe extern "C" fn(*const c_char, *const c_char) -> *mut c_char;
type AddIceCandidateFn = unsafe extern "C" fn(*const c_char, *const c_char, *const c_char) -> c_int;
type RemoveClientFn = unsafe extern "C" fn(*const c_char) -> c_int;
type GetConnectionStateFn = unsafe extern "C" fn(*const c_char) -> c_int;
type GetStatsFn = unsafe extern "C" fn(*mut u64, *mut u64, *mut u64, *mut u64);
type FreeStringFn = unsafe extern "C" fn(*mut c_char);

/// WebRTC library interface - dynamically loaded
pub struct WebRTCLibrary {
    _lib: Library,
    init_service: InitServiceFn,
    shutdown_service: ShutdownServiceFn,
    list_camera_streams: ListCameraStreamsFn,
    register_stream: RegisterStreamFn,
    unregister_stream: UnregisterStreamFn,
    create_client: CreateClientFn,
    set_offer: SetOfferFn,
    add_ice_candidate: AddIceCandidateFn,
    remove_client: RemoveClientFn,
    get_connection_state: GetConnectionStateFn,
    get_stats: GetStatsFn,
    free_string: FreeStringFn,
}

impl WebRTCLibrary {
    /// Load the WebRTC library dynamically
    pub fn load() -> Result<Self> {
        // Try multiple possible library paths
        let library_paths = [
            "/Users/stevengilvarry/Code/zm-next/build/plugins/output_webrtc/output_webrtc.dylib",
            "/opt/homebrew/lib/liboutput_webrtc.dylib",
            "./liboutput_webrtc.dylib",
            "liboutput_webrtc.dylib",
            "output_webrtc",
        ];

        let mut lib = None;
        for path in &library_paths {
            match unsafe { Library::new(path) } {
                Ok(library) => {
                    lib = Some(library);
                    tracing::info!("Successfully loaded WebRTC library from: {}", path);
                    break;
                }
                Err(e) => {
                    tracing::debug!("Failed to load library from {}: {}", path, e);
                    continue;
                }
            }
        }

        let lib = lib.ok_or_else(|| anyhow!("Failed to load WebRTC library from any path"))?;

        // Load all required function symbols
        let init_service: Symbol<InitServiceFn> = unsafe {
            lib.get(b"zm_webrtc_init_service")?
        };
        
        let shutdown_service: Symbol<ShutdownServiceFn> = unsafe {
            lib.get(b"zm_webrtc_shutdown_service")?
        };
        
        let list_camera_streams: Symbol<ListCameraStreamsFn> = unsafe {
            lib.get(b"zm_webrtc_list_camera_streams")?
        };
        
        let register_stream: Symbol<RegisterStreamFn> = unsafe {
            lib.get(b"zm_webrtc_register_stream")?
        };
        
        let unregister_stream: Symbol<UnregisterStreamFn> = unsafe {
            lib.get(b"zm_webrtc_unregister_stream")?
        };
        
        let create_client: Symbol<CreateClientFn> = unsafe {
            lib.get(b"zm_webrtc_create_client")?
        };
        
        let set_offer: Symbol<SetOfferFn> = unsafe {
            lib.get(b"zm_webrtc_set_offer")?
        };
        
        let add_ice_candidate: Symbol<AddIceCandidateFn> = unsafe {
            lib.get(b"zm_webrtc_add_ice_candidate")?
        };
        
        let remove_client: Symbol<RemoveClientFn> = unsafe {
            lib.get(b"zm_webrtc_remove_client")?
        };
        
        let get_connection_state: Symbol<GetConnectionStateFn> = unsafe {
            lib.get(b"zm_webrtc_get_connection_state")?
        };
        
        let get_stats: Symbol<GetStatsFn> = unsafe {
            lib.get(b"zm_webrtc_get_stats")?
        };
        
        let free_string: Symbol<FreeStringFn> = unsafe {
            lib.get(b"zm_webrtc_free_string")?
        };

        // Store function pointers
        let init_service_fn = *init_service;
        let shutdown_service_fn = *shutdown_service;
        let list_camera_streams_fn = *list_camera_streams;
        let register_stream_fn = *register_stream;
        let unregister_stream_fn = *unregister_stream;
        let create_client_fn = *create_client;
        let set_offer_fn = *set_offer;
        let add_ice_candidate_fn = *add_ice_candidate;
        let remove_client_fn = *remove_client;
        let get_connection_state_fn = *get_connection_state;
        let get_stats_fn = *get_stats;
        let free_string_fn = *free_string;

        Ok(WebRTCLibrary {
            _lib: lib,
            init_service: init_service_fn,
            shutdown_service: shutdown_service_fn,
            list_camera_streams: list_camera_streams_fn,
            register_stream: register_stream_fn,
            unregister_stream: unregister_stream_fn,
            create_client: create_client_fn,
            set_offer: set_offer_fn,
            add_ice_candidate: add_ice_candidate_fn,
            remove_client: remove_client_fn,
            get_connection_state: get_connection_state_fn,
            get_stats: get_stats_fn,
            free_string: free_string_fn,
        })
    }

    /// Initialize WebRTC service for a camera
    pub fn init_service(&self, camera_id: u32) -> Result<()> {
        let result = unsafe {
            (self.init_service)(camera_id)
        };
        
        if result != 0 {
            return Err(anyhow!("Failed to initialize WebRTC service: error code {}", result));
        }
        
        Ok(())
    }

    /// Shutdown WebRTC service for a camera
    pub fn shutdown_service(&self, camera_id: u32) -> Result<()> {
        let result = unsafe {
            (self.shutdown_service)(camera_id)
        };
        
        if result != 0 {
            return Err(anyhow!("Failed to shutdown WebRTC service: error code {}", result));
        }
        
        Ok(())
    }

    /// Register a camera stream
    pub fn register_stream(&self, camera_id: u32) -> Result<()> {
        let result = unsafe {
            (self.register_stream)(camera_id)
        };
        
        if result != 0 {
            return Err(anyhow!("Failed to register stream: error code {}", result));
        }
        
        Ok(())
    }

    /// Unregister a camera stream
    pub fn unregister_stream(&self, camera_id: u32) -> Result<()> {
        let result = unsafe {
            (self.unregister_stream)(camera_id)
        };
        
        if result != 0 {
            return Err(anyhow!("Failed to unregister stream: error code {}", result));
        }
        
        Ok(())
    }

    /// Get list of available camera streams
    pub fn list_camera_streams(&self) -> Result<Vec<CameraStream>> {
        let streams_ptr = unsafe { (self.list_camera_streams)() };
        
        if streams_ptr.is_null() {
            return Ok(Vec::new());
        }
        
        let streams_json = unsafe {
            let cstr = CStr::from_ptr(streams_ptr);
            cstr.to_string_lossy().to_string()
        };
        
        unsafe {
            (self.free_string)(streams_ptr);
        }
        
        // Parse JSON response
        let streams: Vec<CameraStream> = serde_json::from_str(&streams_json)
            .unwrap_or_else(|_| Vec::new());
        
        Ok(streams)
    }

    /// Create a new WebRTC client
    pub fn create_client(&self, client_id: &str, camera_id: u32) -> Result<*mut c_void> {
        let client_id_cstr = CString::new(client_id)?;
        
        let client_handle = unsafe {
            (self.create_client)(client_id_cstr.as_ptr(), camera_id)
        };
        
        if client_handle.is_null() {
            return Err(anyhow!("Failed to create WebRTC client"));
        }
        
        Ok(client_handle)
    }

    /// Set remote offer and get answer
    pub fn set_offer(&self, client_id: &str, offer_sdp: &str) -> Result<String> {
        let client_id_cstr = CString::new(client_id)?;
        let offer_cstr = CString::new(offer_sdp)?;
        
        let answer_ptr = unsafe {
            (self.set_offer)(client_id_cstr.as_ptr(), offer_cstr.as_ptr())
        };
        
        if answer_ptr.is_null() {
            return Err(anyhow!("Failed to set offer"));
        }
        
        let answer_sdp = unsafe {
            let cstr = CStr::from_ptr(answer_ptr);
            cstr.to_string_lossy().to_string()
        };
        
        unsafe {
            (self.free_string)(answer_ptr);
        }
        
        Ok(answer_sdp)
    }

    /// Add ICE candidate
    pub fn add_ice_candidate(&self, client_id: &str, candidate: &IceCandidate) -> Result<()> {
        let client_id_cstr = CString::new(client_id)?;
        let candidate_cstr = CString::new(candidate.candidate.clone())?;
        let sdp_mid_cstr = candidate.sdp_mid.as_ref()
            .map(|mid| CString::new(mid.clone()))
            .transpose()?;
        
        let sdp_mid_ptr = sdp_mid_cstr.as_ref()
            .map(|cstr| cstr.as_ptr())
            .unwrap_or(std::ptr::null());
        
        let result = unsafe {
            (self.add_ice_candidate)(
                client_id_cstr.as_ptr(),
                candidate_cstr.as_ptr(),
                sdp_mid_ptr,
            )
        };
        
        if result != 0 {
            return Err(anyhow!("Failed to add ICE candidate: error code {}", result));
        }
        
        Ok(())
    }

    /// Remove client
    pub fn remove_client(&self, client_id: &str) -> Result<()> {
        let client_id_cstr = CString::new(client_id)?;
        let result = unsafe {
            (self.remove_client)(client_id_cstr.as_ptr())
        };
        
        if result != 0 {
            return Err(anyhow!("Failed to remove client: error code {}", result));
        }
        
        Ok(())
    }

    /// Get connection state
    pub fn get_connection_state(&self, client_id: &str) -> Result<SessionState> {
        let client_id_cstr = CString::new(client_id)?;
        let state = unsafe { 
            (self.get_connection_state)(client_id_cstr.as_ptr()) 
        };
        
        if state < 0 {
            return Err(anyhow!("Failed to get connection state: error code {}", state));
        }
        
        Ok(SessionState::from(state))
    }

    /// Get global statistics (total frames, bytes, clients connected/disconnected)
    pub fn get_global_stats(&self) -> Result<(u64, u64, u64, u64)> {
        let mut total_frames: u64 = 0;
        let mut total_bytes: u64 = 0;
        let mut clients_connected: u64 = 0;
        let mut clients_disconnected: u64 = 0;
        
        unsafe {
            (self.get_stats)(
                &mut total_frames,
                &mut total_bytes,
                &mut clients_connected,
                &mut clients_disconnected,
            );
        }
        
        Ok((total_frames, total_bytes, clients_connected, clients_disconnected))
    }
}

/// Global WebRTC library instance
static WEBRTC_LIB: OnceCell<Arc<Mutex<WebRTCLibrary>>> = OnceCell::new();

/// Get or initialize the global WebRTC library instance
pub fn get_webrtc_lib() -> Result<Arc<Mutex<WebRTCLibrary>>> {
    WEBRTC_LIB.get_or_try_init(|| {
        let lib = WebRTCLibrary::load()?;
        Ok(Arc::new(Mutex::new(lib)))
    }).cloned()
}

/// WebRTC client session - now uses dynamic library
#[derive(Debug)]
pub struct WebRTCSession {
    client_id: String,
    #[allow(dead_code)]
    camera_id: u32,
}

impl WebRTCSession {
    /// Create a new WebRTC session
    pub fn new(camera_id: u32) -> Result<Self> {
        let client_id = format!("client_{}", Uuid::new_v4());
        
        let lib = get_webrtc_lib()?;
        let lib = lib.lock().map_err(|_| anyhow!("Failed to lock WebRTC library"))?;
        
        // Create the client (this returns a handle but we'll use client_id for subsequent calls)
        let _client_handle = lib.create_client(&client_id, camera_id)?;
        
        Ok(WebRTCSession {
            client_id,
            camera_id,
        })
    }
    
    /// Set the remote offer and get the answer SDP
    pub fn set_offer(&self, offer_sdp: &str) -> Result<String> {
        let lib = get_webrtc_lib()?;
        let lib = lib.lock().map_err(|_| anyhow!("Failed to lock WebRTC library"))?;
        
        lib.set_offer(&self.client_id, offer_sdp)
    }
    
    /// Add an ICE candidate
    pub fn add_ice_candidate(&self, candidate: &IceCandidate) -> Result<()> {
        let lib = get_webrtc_lib()?;
        let lib = lib.lock().map_err(|_| anyhow!("Failed to lock WebRTC library"))?;
        
        lib.add_ice_candidate(&self.client_id, candidate)
    }
    
    /// Get the current connection state
    pub fn get_connection_state(&self) -> Result<SessionState> {
        let lib = get_webrtc_lib()?;
        let lib = lib.lock().map_err(|_| anyhow!("Failed to lock WebRTC library"))?;
        
        lib.get_connection_state(&self.client_id)
    }
    
    /// Get global statistics (not per-client in this implementation)
    pub fn get_stats(&self) -> Result<String> {
        let lib = get_webrtc_lib()?;
        let lib = lib.lock().map_err(|_| anyhow!("Failed to lock WebRTC library"))?;
        
        let (total_frames, total_bytes, clients_connected, clients_disconnected) = lib.get_global_stats()?;
        
        // Return stats as JSON
        let stats = serde_json::json!({
            "total_frames": total_frames,
            "total_bytes": total_bytes,
            "clients_connected": clients_connected,
            "clients_disconnected": clients_disconnected,
            "client_id": self.client_id
        });
        
        Ok(stats.to_string())
    }
    
    /// Get the client ID
    pub fn client_id(&self) -> &str {
        &self.client_id
    }
}

impl Drop for WebRTCSession {
    fn drop(&mut self) {
        if let Ok(lib) = get_webrtc_lib() {
            if let Ok(lib) = lib.lock() {
                let _ = lib.remove_client(&self.client_id);
            }
        }
    }
}

unsafe impl Send for WebRTCSession {}
unsafe impl Sync for WebRTCSession {}

/// Initialize the WebRTC service for a camera
pub fn init_webrtc_service(camera_id: u32) -> Result<()> {
    let lib = get_webrtc_lib()?;
    let lib = lib.lock().map_err(|_| anyhow!("Failed to lock WebRTC library"))?;
    
    lib.init_service(camera_id)
}

/// Get list of available camera streams
pub fn list_camera_streams() -> Result<Vec<CameraStream>> {
    let lib = get_webrtc_lib()?;
    let lib = lib.lock().map_err(|_| anyhow!("Failed to lock WebRTC library"))?;
    
    lib.list_camera_streams()
}

/// Get statistics from the running WebRTC service
pub fn get_webrtc_stats() -> Result<WebRTCStats> {
    let lib = get_webrtc_lib()?;
    let lib = lib.lock().map_err(|_| anyhow!("Failed to lock WebRTC library"))?;
    
    let (total_frames, total_bytes, clients_connected, clients_disconnected) = lib.get_global_stats()?;
    
    Ok(WebRTCStats {
        total_frames,
        total_bytes,
        clients_connected,
        clients_disconnected,
    })
}

/// Create a WebRTC client for a specific camera (works with running service)
pub fn create_webrtc_client(client_id: &str, camera_id: u32) -> Result<()> {
    let lib = get_webrtc_lib()?;
    let lib = lib.lock().map_err(|_| anyhow!("Failed to lock WebRTC library"))?;
    
    lib.create_client(client_id, camera_id)?;
    Ok(())
}

/// Get connection state for a specific client
pub fn get_client_connection_state(client_id: &str) -> Result<SessionState> {
    let lib = get_webrtc_lib()?;
    let lib = lib.lock().map_err(|_| anyhow!("Failed to lock WebRTC library"))?;
    
    lib.get_connection_state(client_id)
}

/// Remove a WebRTC client
pub fn remove_webrtc_client(client_id: &str) -> Result<()> {
    let lib = get_webrtc_lib()?;
    let lib = lib.lock().map_err(|_| anyhow!("Failed to lock WebRTC library"))?;
    
    lib.remove_client(client_id)?;
    Ok(())
}

/// Set offer for a WebRTC client and get answer
pub fn set_webrtc_offer(client_id: &str, offer_sdp: &str) -> Result<String> {
    let lib = get_webrtc_lib()?;
    let lib = lib.lock().map_err(|_| anyhow!("Failed to lock WebRTC library"))?;
    
    lib.set_offer(client_id, offer_sdp)
}

/// Add ICE candidate for a WebRTC client
pub fn add_webrtc_ice_candidate(client_id: &str, candidate: &IceCandidate) -> Result<()> {
    let lib = get_webrtc_lib()?;
    let lib = lib.lock().map_err(|_| anyhow!("Failed to lock WebRTC library"))?;
    
    lib.add_ice_candidate(client_id, candidate)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_session_state_conversion() {
        assert!(matches!(SessionState::from(0), SessionState::New));
        assert!(matches!(SessionState::from(2), SessionState::Connected));
        assert!(matches!(SessionState::from(99), SessionState::Failed));
    }
    
    #[test]
    fn test_ice_candidate_creation() {
        let candidate = IceCandidate {
            candidate: "candidate:1 1 UDP 2130706431 192.168.1.100 54400 typ host".to_string(),
            sdp_mid: Some("0".to_string()),
            sdp_mline_index: Some(0),
        };
        
        assert_eq!(candidate.candidate, "candidate:1 1 UDP 2130706431 192.168.1.100 54400 typ host");
        assert_eq!(candidate.sdp_mid, Some("0".to_string()));
        assert_eq!(candidate.sdp_mline_index, Some(0));
    }
}
