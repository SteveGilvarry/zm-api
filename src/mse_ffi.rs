/// FFI bindings for the C++ MSE output plugin
/// Located at: plugins/output_mse/mse_api.h
/// 
/// ⚠️  DEPRECATED: The MSE plugin architecture has changed from FFI to socket-based communication.
/// The plugin no longer exposes C functions for external access. All zm_mse_* FFI functions
/// have been removed from the plugin. Use the new socket-based communication via MseSocketClient
/// in mse_socket_client.rs instead.
/// 
/// New communication method:
/// - TCP server on 127.0.0.1:9051
/// - JSON command protocol  
/// - No more direct memory access via pointers
/// - Multi-stream support under single service instance

// Conditional compilation based on whether the MSE library is available
#[cfg(feature = "mse_plugin")]
use std::ffi::c_char;

#[cfg(feature = "mse_plugin")]
#[link(name = "output_mse")]
extern "C" {
    /// Register a video stream (call once per camera)
    pub fn zm_mse_register_stream(
        camera_id: u32,
        stream_id: u32,
        codec: *const c_char,
        width: i32,
        height: i32,
    );

    /// Unregister when done
    pub fn zm_mse_unregister_stream(camera_id: u32, stream_id: u32);

    /// Push a segment to the MSE buffer (used by ZoneMinder)
    pub fn zm_mse_push_segment(
        camera_id: u32,
        data: *const u8,
        size: usize,
    ) -> i32;

    /// Get next fMP4 segment (BLOCKING - use in dedicated thread)
    pub fn zm_mse_pop_segment(
        camera_id: u32,
        buffer: *mut u8,
        max_size: usize,
    ) -> usize;

    /// Non-blocking version (returns 0 if no segment available)
    pub fn zm_mse_try_pop_segment(
        camera_id: u32,
        buffer: *mut u8,
        max_size: usize,
    ) -> usize;

    /// Get current buffer status
    pub fn zm_mse_get_buffer_size(camera_id: u32) -> usize;

    /// Get detailed statistics
    pub fn zm_mse_get_buffer_stats(
        camera_id: u32,
        total_segments: *mut u64,
        dropped_segments: *mut u64,
    ) -> usize;

    /// Get stream metrics
    pub fn zm_mse_get_bytes_received(camera_id: u32) -> u64;
    pub fn zm_mse_get_frame_count(camera_id: u32) -> u64;

    /// Get initialization segment
    pub fn zm_mse_get_init_segment(
        camera_id: u32,
        buffer: *mut u8,
        max_size: usize,
    ) -> usize;

    /// Get latest segment
    pub fn zm_mse_get_latest_segment(
        camera_id: u32,
        buffer: *mut u8,
        max_size: usize,
    ) -> usize;
}

// Mock implementation for testing without the actual MSE plugin
#[cfg(not(feature = "mse_plugin"))]
pub mod mock {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::time::{SystemTime, UNIX_EPOCH};
    use tracing::{debug, warn};

    // Mock state for testing
    lazy_static::lazy_static! {
        static ref MOCK_STATE: Arc<Mutex<MockMseState>> = Arc::new(Mutex::new(MockMseState::new()));
    }

    struct MockCamera {
        camera_id: u32,
        stream_id: u32,
        width: i32,
        height: i32,
        segment_count: u64,
        bytes_received: u64,
        frame_count: u64,
        last_segment_time: u64,
    }

    struct MockMseState {
        cameras: HashMap<u32, MockCamera>,
    }

    impl MockMseState {
        fn new() -> Self {
            Self {
                cameras: HashMap::new(),
            }
        }
    }

    // Generate a mock fMP4 initialization segment
    fn generate_mock_init_segment() -> Vec<u8> {
        // This is a simplified mock init segment
        // In reality, this would be a proper fMP4 init segment with ftyp and moov boxes
        let mut segment = Vec::new();
        
        // ftyp box (simplified)
        segment.extend_from_slice(b"\x00\x00\x00\x18"); // box size
        segment.extend_from_slice(b"ftyp"); // box type
        segment.extend_from_slice(b"isom"); // major brand
        segment.extend_from_slice(b"\x00\x00\x02\x00"); // minor version
        segment.extend_from_slice(b"isom\x61\x76\x63\x31"); // compatible brands
        
        // moov box (simplified - would contain track info in reality)
        segment.extend_from_slice(b"\x00\x00\x00\x08"); // box size
        segment.extend_from_slice(b"moov"); // box type
        
        segment
    }

    // Generate a mock fMP4 media segment
    fn generate_mock_media_segment(sequence: u64) -> Vec<u8> {
        let mut segment = Vec::new();
        
        // moof box (simplified)
        segment.extend_from_slice(b"\x00\x00\x00\x20"); // box size
        segment.extend_from_slice(b"moof"); // box type
        
        // mfhd box
        segment.extend_from_slice(b"\x00\x00\x00\x10"); // box size
        segment.extend_from_slice(b"mfhd"); // box type
        segment.extend_from_slice(b"\x00\x00\x00\x00"); // version + flags
        segment.extend_from_slice(&(sequence as u32).to_be_bytes()); // sequence number
        
        // traf box (simplified)
        segment.extend_from_slice(b"\x00\x00\x00\x08"); // box size
        segment.extend_from_slice(b"traf"); // box type
        
        // mdat box with some mock H.264 data
        let mdat_size = 1024u32; // 1KB of mock data
        segment.extend_from_slice(&(mdat_size + 8).to_be_bytes()); // box size
        segment.extend_from_slice(b"mdat"); // box type
        
        // Mock H.264 NAL units (simplified)
        for _ in 0..mdat_size {
            segment.push((sequence % 256) as u8); // Some varying data based on sequence
        }
        
        segment
    }

    pub unsafe fn zm_mse_register_stream(
        camera_id: u32,
        stream_id: u32,
        _codec: *const std::ffi::c_char,
        width: i32,
        height: i32,
    ) {
        debug!("Mock: Registering stream for camera {} ({}x{})", camera_id, width, height);
        
        let mut state = MOCK_STATE.lock().unwrap();
        state.cameras.insert(camera_id, MockCamera {
            camera_id,
            stream_id,
            width,
            height,
            segment_count: 0,
            bytes_received: 0,
            frame_count: 0,
            last_segment_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
        });
    }

    pub unsafe fn zm_mse_unregister_stream(camera_id: u32, _stream_id: u32) {
        debug!("Mock: Unregistering stream for camera {}", camera_id);
        
        let mut state = MOCK_STATE.lock().unwrap();
        state.cameras.remove(&camera_id);
    }

    pub unsafe fn zm_mse_pop_segment(
        camera_id: u32,
        buffer: *mut u8,
        max_size: usize,
    ) -> usize {
        // This would normally block, but for mock we'll return immediately
        zm_mse_try_pop_segment(camera_id, buffer, max_size)
    }

    pub unsafe fn zm_mse_try_pop_segment(
        camera_id: u32,
        buffer: *mut u8,
        max_size: usize,
    ) -> usize {
        let mut state = MOCK_STATE.lock().unwrap();
        
        if let Some(camera) = state.cameras.get_mut(&camera_id) {
            let segment = if camera.segment_count == 0 {
                // First segment is init segment
                generate_mock_init_segment()
            } else {
                // Subsequent segments are media segments
                generate_mock_media_segment(camera.segment_count)
            };
            
            if segment.len() <= max_size {
                std::ptr::copy_nonoverlapping(segment.as_ptr(), buffer, segment.len());
                
                camera.segment_count += 1;
                camera.bytes_received += segment.len() as u64;
                camera.frame_count += 1;
                camera.last_segment_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
                
                debug!("Mock: Generated segment {} for camera {} ({} bytes)", 
                       camera.segment_count - 1, camera_id, segment.len());
                
                segment.len()
            } else {
                warn!("Mock: Buffer too small for segment: need {}, got {}", segment.len(), max_size);
                0
            }
        } else {
            debug!("Mock: Camera {} not registered", camera_id);
            0
        }
    }

    pub unsafe fn zm_mse_get_buffer_size(camera_id: u32) -> usize {
        let state = MOCK_STATE.lock().unwrap();
        if state.cameras.contains_key(&camera_id) {
            10 // Mock buffer size
        } else {
            0
        }
    }

    pub unsafe fn zm_mse_get_buffer_stats(
        camera_id: u32,
        total_segments: *mut u64,
        dropped_segments: *mut u64,
    ) -> usize {
        let state = MOCK_STATE.lock().unwrap();
        if let Some(camera) = state.cameras.get(&camera_id) {
            *total_segments = camera.segment_count;
            *dropped_segments = 0; // Mock: no dropped segments
            camera.segment_count as usize
        } else {
            *total_segments = 0;
            *dropped_segments = 0;
            0
        }
    }

    pub unsafe fn zm_mse_get_bytes_received(camera_id: u32) -> u64 {
        let state = MOCK_STATE.lock().unwrap();
        state.cameras.get(&camera_id).map(|c| c.bytes_received).unwrap_or(0)
    }

    pub unsafe fn zm_mse_get_frame_count(camera_id: u32) -> u64 {
        let state = MOCK_STATE.lock().unwrap();
        state.cameras.get(&camera_id).map(|c| c.frame_count).unwrap_or(0)
    }
}

// Export the appropriate functions based on feature flags
#[cfg(not(feature = "mse_plugin"))]
pub use self::mock::{
    zm_mse_register_stream, zm_mse_unregister_stream, zm_mse_pop_segment,
    zm_mse_try_pop_segment, zm_mse_get_buffer_size, zm_mse_get_buffer_stats,
    zm_mse_get_bytes_received, zm_mse_get_frame_count,
};
