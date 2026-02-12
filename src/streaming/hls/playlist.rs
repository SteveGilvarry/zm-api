//! M3U8 playlist generation for HLS streaming
//!
//! Generates both master playlists (for ABR) and media playlists (segment lists).

use std::fmt::Write;

/// Quality variant for ABR streaming
#[derive(Debug, Clone)]
pub struct QualityVariant {
    pub name: String,
    pub bandwidth: u64,
    pub resolution: Option<(u32, u32)>,
    pub codecs: Option<String>,
    pub frame_rate: Option<f32>,
}

impl Default for QualityVariant {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            bandwidth: 2_500_000,
            resolution: Some((1280, 720)),
            codecs: Some("avc1.42e01e,mp4a.40.2".to_string()),
            frame_rate: Some(30.0),
        }
    }
}

/// Master playlist for ABR (Adaptive Bitrate) streaming
#[derive(Debug, Clone)]
pub struct MasterPlaylist {
    pub variants: Vec<QualityVariant>,
    pub base_url: String,
}

impl MasterPlaylist {
    /// Create a new master playlist
    pub fn new(base_url: &str) -> Self {
        Self {
            variants: Vec::new(),
            base_url: base_url.to_string(),
        }
    }

    /// Add a quality variant
    pub fn add_variant(&mut self, variant: QualityVariant) {
        self.variants.push(variant);
    }

    /// Generate the M3U8 content
    pub fn generate(&self) -> String {
        let mut output = String::new();

        writeln!(output, "#EXTM3U").unwrap();
        writeln!(output, "#EXT-X-VERSION:6").unwrap();

        for variant in &self.variants {
            // Build stream info line
            let mut stream_info = format!("BANDWIDTH={}", variant.bandwidth);

            if let Some((width, height)) = variant.resolution {
                write!(stream_info, ",RESOLUTION={}x{}", width, height).unwrap();
            }

            if let Some(ref codecs) = variant.codecs {
                write!(stream_info, ",CODECS=\"{}\"", codecs).unwrap();
            }

            if let Some(frame_rate) = variant.frame_rate {
                write!(stream_info, ",FRAME-RATE={:.3}", frame_rate).unwrap();
            }

            writeln!(output, "#EXT-X-STREAM-INF:{}", stream_info).unwrap();
            writeln!(output, "{}/{}.m3u8", self.base_url, variant.name).unwrap();
        }

        output
    }
}

/// Segment reference in a media playlist
#[derive(Debug, Clone)]
pub struct SegmentRef {
    pub sequence: u64,
    pub duration: f64,
    pub uri: String,
    pub is_independent: bool,
    /// For LL-HLS: partial segments
    pub parts: Vec<PartialSegmentRef>,
}

/// Partial segment reference for LL-HLS
#[derive(Debug, Clone)]
pub struct PartialSegmentRef {
    pub duration: f64,
    pub uri: String,
    pub is_independent: bool,
}

/// Media playlist (variant playlist with segment list)
#[derive(Debug, Clone)]
pub struct MediaPlaylist {
    pub target_duration: u32,
    pub media_sequence: u64,
    pub discontinuity_sequence: u64,
    pub segments: Vec<SegmentRef>,
    pub init_segment_uri: Option<String>,
    pub is_live: bool,
    /// LL-HLS: part target duration
    pub part_target_duration: Option<f64>,
    /// LL-HLS: server control parameters
    pub server_control: Option<ServerControl>,
}

/// LL-HLS server control parameters
#[derive(Debug, Clone)]
pub struct ServerControl {
    pub can_block_reload: bool,
    pub part_hold_back: f64,
    pub can_skip_until: Option<f64>,
}

impl Default for MediaPlaylist {
    fn default() -> Self {
        Self {
            target_duration: 4,
            media_sequence: 0,
            discontinuity_sequence: 0,
            segments: Vec::new(),
            init_segment_uri: None,
            is_live: true,
            part_target_duration: None,
            server_control: None,
        }
    }
}

impl MediaPlaylist {
    /// Create a new media playlist
    pub fn new(target_duration: u32) -> Self {
        Self {
            target_duration,
            ..Default::default()
        }
    }

    /// Create an LL-HLS enabled playlist
    pub fn new_ll_hls(target_duration: u32, part_target_duration: f64) -> Self {
        Self {
            target_duration,
            part_target_duration: Some(part_target_duration),
            server_control: Some(ServerControl {
                can_block_reload: true,
                part_hold_back: part_target_duration * 3.0,
                can_skip_until: Some(target_duration as f64 * 6.0),
            }),
            ..Default::default()
        }
    }

    /// Add a segment to the playlist
    pub fn add_segment(&mut self, segment: SegmentRef) {
        self.segments.push(segment);
    }

    /// Set the initialization segment URI
    pub fn set_init_segment(&mut self, uri: &str) {
        self.init_segment_uri = Some(uri.to_string());
    }

    /// Get the current media sequence number
    pub fn current_sequence(&self) -> u64 {
        self.media_sequence + self.segments.len() as u64
    }

    /// Remove old segments to maintain playlist size
    pub fn trim_to_size(&mut self, max_segments: usize) {
        if self.segments.len() > max_segments {
            let remove_count = self.segments.len() - max_segments;
            self.segments.drain(0..remove_count);
            self.media_sequence += remove_count as u64;
        }
    }

    /// Generate the M3U8 content
    pub fn generate(&self) -> String {
        let mut output = String::new();

        writeln!(output, "#EXTM3U").unwrap();
        writeln!(output, "#EXT-X-VERSION:6").unwrap();
        writeln!(output, "#EXT-X-TARGETDURATION:{}", self.target_duration).unwrap();
        writeln!(output, "#EXT-X-MEDIA-SEQUENCE:{}", self.media_sequence).unwrap();

        if self.discontinuity_sequence > 0 {
            writeln!(
                output,
                "#EXT-X-DISCONTINUITY-SEQUENCE:{}",
                self.discontinuity_sequence
            )
            .unwrap();
        }

        // LL-HLS specific tags
        if let Some(part_duration) = self.part_target_duration {
            writeln!(output, "#EXT-X-PART-INF:PART-TARGET={:.3}", part_duration).unwrap();
        }

        if let Some(ref server_control) = self.server_control {
            let mut control = String::new();
            if server_control.can_block_reload {
                write!(control, "CAN-BLOCK-RELOAD=YES").unwrap();
            }
            write!(
                control,
                ",PART-HOLD-BACK={:.3}",
                server_control.part_hold_back
            )
            .unwrap();
            if let Some(skip_until) = server_control.can_skip_until {
                write!(control, ",CAN-SKIP-UNTIL={:.3}", skip_until).unwrap();
            }
            writeln!(output, "#EXT-X-SERVER-CONTROL:{}", control).unwrap();
        }

        // Initialization segment (fMP4)
        if let Some(ref init_uri) = self.init_segment_uri {
            writeln!(output, "#EXT-X-MAP:URI=\"{}\"", init_uri).unwrap();
        }

        // Segments
        for segment in &self.segments {
            // LL-HLS partial segments
            for part in &segment.parts {
                let mut part_info = format!("DURATION={:.3},URI=\"{}\"", part.duration, part.uri);
                if part.is_independent {
                    write!(part_info, ",INDEPENDENT=YES").unwrap();
                }
                writeln!(output, "#EXT-X-PART:{}", part_info).unwrap();
            }

            // Full segment
            if segment.is_independent {
                writeln!(output, "#EXT-X-INDEPENDENT-SEGMENTS").unwrap();
            }
            writeln!(output, "#EXTINF:{:.3},", segment.duration).unwrap();
            writeln!(output, "{}", segment.uri).unwrap();
        }

        // End tag for VOD
        if !self.is_live {
            writeln!(output, "#EXT-X-ENDLIST").unwrap();
        }

        output
    }
}

/// Playlist generator that manages master and media playlists
pub struct PlaylistGenerator {
    pub monitor_id: u32,
    pub base_url: String,
    pub target_duration: u32,
    pub playlist_size: usize,
    pub ll_hls_enabled: bool,
    pub part_target_duration: f64,
}

impl PlaylistGenerator {
    /// Create a new playlist generator
    pub fn new(
        monitor_id: u32,
        base_url: &str,
        target_duration: u32,
        playlist_size: usize,
    ) -> Self {
        Self {
            monitor_id,
            base_url: base_url.to_string(),
            target_duration,
            playlist_size,
            ll_hls_enabled: false,
            part_target_duration: 0.3,
        }
    }

    /// Enable LL-HLS with specified part duration
    pub fn with_ll_hls(mut self, part_target_duration: f64) -> Self {
        self.ll_hls_enabled = true;
        self.part_target_duration = part_target_duration;
        self
    }

    /// Generate a master playlist with default variant
    pub fn generate_master_playlist(&self) -> MasterPlaylist {
        let mut master = MasterPlaylist::new(&self.base_url);

        // Add default passthrough variant
        master.add_variant(QualityVariant {
            name: "live".to_string(),
            bandwidth: 5_000_000,
            resolution: None, // Will be detected from stream
            codecs: Some("avc1.42e01e".to_string()),
            frame_rate: None,
        });

        master
    }

    /// Generate a media playlist
    pub fn generate_media_playlist(&self) -> MediaPlaylist {
        let mut playlist = if self.ll_hls_enabled {
            MediaPlaylist::new_ll_hls(self.target_duration, self.part_target_duration)
        } else {
            MediaPlaylist::new(self.target_duration)
        };

        playlist.set_init_segment(&format!("{}/init.mp4", self.base_url));

        playlist
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_master_playlist_generation() {
        let mut master = MasterPlaylist::new("/hls/1");

        master.add_variant(QualityVariant {
            name: "720p".to_string(),
            bandwidth: 2_500_000,
            resolution: Some((1280, 720)),
            codecs: Some("avc1.42e01e,mp4a.40.2".to_string()),
            frame_rate: Some(30.0),
        });

        master.add_variant(QualityVariant {
            name: "1080p".to_string(),
            bandwidth: 5_000_000,
            resolution: Some((1920, 1080)),
            codecs: Some("avc1.640028,mp4a.40.2".to_string()),
            frame_rate: Some(30.0),
        });

        let content = master.generate();

        assert!(content.contains("#EXTM3U"));
        assert!(content.contains("#EXT-X-VERSION:6"));
        assert!(content.contains("BANDWIDTH=2500000"));
        assert!(content.contains("RESOLUTION=1280x720"));
        assert!(content.contains("/hls/1/720p.m3u8"));
        assert!(content.contains("/hls/1/1080p.m3u8"));
    }

    #[test]
    fn test_media_playlist_generation() {
        let mut playlist = MediaPlaylist::new(4);
        playlist.set_init_segment("init.mp4");

        playlist.add_segment(SegmentRef {
            sequence: 0,
            duration: 4.0,
            uri: "segment_00000.m4s".to_string(),
            is_independent: true,
            parts: vec![],
        });

        playlist.add_segment(SegmentRef {
            sequence: 1,
            duration: 4.0,
            uri: "segment_00001.m4s".to_string(),
            is_independent: false,
            parts: vec![],
        });

        let content = playlist.generate();

        assert!(content.contains("#EXTM3U"));
        assert!(content.contains("#EXT-X-TARGETDURATION:4"));
        assert!(content.contains("#EXT-X-MEDIA-SEQUENCE:0"));
        assert!(content.contains("#EXT-X-MAP:URI=\"init.mp4\""));
        assert!(content.contains("#EXTINF:4.000,"));
        assert!(content.contains("segment_00000.m4s"));
        assert!(content.contains("segment_00001.m4s"));
        // Live playlist should not have ENDLIST
        assert!(!content.contains("#EXT-X-ENDLIST"));
    }

    #[test]
    fn test_ll_hls_playlist_generation() {
        let mut playlist = MediaPlaylist::new_ll_hls(4, 0.3);
        playlist.set_init_segment("init.mp4");

        playlist.add_segment(SegmentRef {
            sequence: 0,
            duration: 4.0,
            uri: "segment_00000.m4s".to_string(),
            is_independent: true,
            parts: vec![
                PartialSegmentRef {
                    duration: 0.3,
                    uri: "segment_00000.0.m4s".to_string(),
                    is_independent: true,
                },
                PartialSegmentRef {
                    duration: 0.3,
                    uri: "segment_00000.1.m4s".to_string(),
                    is_independent: false,
                },
            ],
        });

        let content = playlist.generate();

        assert!(content.contains("#EXT-X-PART-INF:PART-TARGET=0.300"));
        assert!(content.contains("#EXT-X-SERVER-CONTROL:CAN-BLOCK-RELOAD=YES"));
        assert!(content.contains("#EXT-X-PART:DURATION=0.300"));
        assert!(content.contains("INDEPENDENT=YES"));
    }

    #[test]
    fn test_playlist_trim() {
        let mut playlist = MediaPlaylist::new(4);

        for i in 0..10 {
            playlist.add_segment(SegmentRef {
                sequence: i,
                duration: 4.0,
                uri: format!("segment_{:05}.m4s", i),
                is_independent: i == 0,
                parts: vec![],
            });
        }

        assert_eq!(playlist.segments.len(), 10);
        assert_eq!(playlist.media_sequence, 0);

        playlist.trim_to_size(6);

        assert_eq!(playlist.segments.len(), 6);
        assert_eq!(playlist.media_sequence, 4);
    }

    #[test]
    fn test_playlist_generator() {
        let generator = PlaylistGenerator::new(1, "/api/v3/live/1/hls", 4, 6);

        let master = generator.generate_master_playlist();
        assert_eq!(master.variants.len(), 1);
        assert_eq!(master.variants[0].name, "live");

        let master_content = master.generate();
        assert!(master_content.contains("/api/v3/live/1/hls/live.m3u8"));

        let media = generator.generate_media_playlist();
        assert_eq!(media.target_duration, 4);
        assert_eq!(
            media.init_segment_uri.as_deref(),
            Some("/api/v3/live/1/hls/init.mp4")
        );
    }
}
