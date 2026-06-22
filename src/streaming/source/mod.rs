pub mod media;
pub mod protocol;
pub mod router;
pub mod stream_socket;

// Re-export media types and bitstream helpers
pub use media::{
    extract_profile_level_id, h264_nal_type, h265_nal_type, slice_starts_picture, AdtsHeader,
    AudioCodec, AudioPacket, VideoCodec, VideoPacket,
};

// Re-export stream-socket reader types
pub use stream_socket::{stream_socket_path, SocketEvent, SourceError, StreamSocketReader};

// Re-export the monitor EVENT type for DB ingest consumers.
pub use protocol::MonitorEvent;

// Re-export router types
pub use router::{
    CachedKeyframe, MonitorEventEnvelope, MonitorSource, ReaderHealth, RouterConfig, RouterError,
    SourceRouter, SourceStats, StreamInfo,
};
