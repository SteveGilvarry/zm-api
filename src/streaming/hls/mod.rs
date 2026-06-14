//! Native HLS (HTTP Live Streaming) implementation
//!
//! This module provides native HLS streaming capabilities including:
//! - fMP4 segment generation from H.264/H.265 NAL units
//! - M3U8 playlist generation (master and variant playlists)
//! - Low-latency HLS (LL-HLS) support with partial segments
//! - Segment storage management with automatic cleanup
//!
//! # Architecture
//!
//! ```text
//! Stream Socket Source → Segmenter → Storage → HTTP Handlers
//!                         ↓
//!                   Playlist Generator
//! ```

pub mod h264;
pub mod playlist;
pub mod segmenter;
pub mod session;
pub mod storage;
pub mod vod;

pub use playlist::{MasterPlaylist, MediaPlaylist, PlaylistGenerator};
pub use segmenter::{FMP4Segment, HlsSegmenter, InitSegment};
pub use session::{HlsError, HlsSession, HlsSessionManager, HlsSessionStats};
pub use storage::{HlsStorage, SegmentInfo};
