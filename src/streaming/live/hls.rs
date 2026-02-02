//! HLS-specific live streaming functionality
//!
//! This module provides HLS streaming from FIFO sources. It uses the
//! existing HlsSessionManager and HlsSegmenter infrastructure but
//! coordinates them through the LiveStreamCoordinator.

// The HLS functionality is primarily handled by:
// - `crate::streaming::hls::HlsSessionManager` - session lifecycle
// - `crate::streaming::hls::HlsSegmenter` - fMP4 segment generation
// - `crate::streaming::hls::HlsStorage` - segment storage
//
// This module re-exports key types for convenience and may contain
// HLS-specific extensions in the future.

pub use crate::streaming::hls::{HlsError, HlsSession, HlsSessionManager, HlsSessionStats};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hls_reexports() {
        // Just verify the re-exports compile
        fn _takes_manager(_m: &HlsSessionManager) {}
    }
}
