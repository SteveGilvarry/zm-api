//! Wire protocol for ZoneMinder's per-monitor stream socket.
//!
//! Each `zmc` process serves the monitor's compressed media on
//! `{PATH_SOCKS}/stream_{monitor_id}.sock` (see `docs/stream_socket.rst` in
//! the ZoneMinder tree). All integers are little-endian. Every message starts
//! with a 24-byte fixed header:
//!
//! ```text
//! u32  length      bytes following this field (20 + payload size)
//! u8   version     protocol version, 1
//! u8   type        message type
//! u8   stream      0 = video, 1 = audio
//! u8   flags       bit 0: keyframe (video); other bits reserved
//! u32  sequence    per-stream, counts every message produced
//! u32  generation  stream epoch; a bump means re-init the decoder
//! u64  pts_us      microseconds (AV_TIME_BASE_Q), shared clock
//! ```
//!
//! Version 1 has no client-to-server messages; zmc ignores inbound bytes.

use super::media::{AudioCodec, VideoCodec};

pub const PROTOCOL_VERSION: u8 = 1;

/// Total serialized header size, including the leading length field.
pub const HEADER_SIZE: usize = 24;

/// Bytes of fixed header counted by the `length` field (everything after the
/// length field itself).
const HEADER_LENGTH_BYTES: u32 = (HEADER_SIZE - 4) as u32;

/// Sanity cap on the length field; larger values mean a corrupt or hostile
/// peer (matches zmc's `kMaxMessageLength`).
pub const MAX_MESSAGE_LENGTH: u32 = 32 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Hello,
    Media,
    Keyframe,
    Stats,
    Bye,
}

impl MessageType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x01 => Some(MessageType::Hello),
            0x02 => Some(MessageType::Media),
            0x03 => Some(MessageType::Keyframe),
            0x04 => Some(MessageType::Stats),
            0x05 => Some(MessageType::Bye),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamId {
    Video,
    Audio,
}

impl StreamId {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(StreamId::Video),
            1 => Some(StreamId::Audio),
            _ => None,
        }
    }
}

/// Header flag: the message carries a video keyframe access unit.
pub const FLAG_KEYFRAME: u8 = 0x01;

// HELLO TLV tags
const TLV_CODEC_ID: u8 = 0x01; // u32, AVCodecID value
const TLV_EXTRADATA: u8 = 0x02; // raw codecpar->extradata
const TLV_WIDTH: u8 = 0x03; // u32
const TLV_HEIGHT: u8 = 0x04; // u32
const TLV_FPS_NUM: u8 = 0x05; // u32
const TLV_FPS_DEN: u8 = 0x06; // u32
const TLV_SAMPLE_RATE: u8 = 0x07; // u32
const TLV_CHANNELS: u8 = 0x08; // u32
const TLV_PROFILE: u8 = 0x09; // u32
const TLV_LEVEL: u8 = 0x0A; // u32

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ProtocolError {
    #[error("unsupported stream socket protocol version {0}")]
    UnsupportedVersion(u8),

    #[error("implausible message length {0}")]
    BadLength(u32),

    #[error("truncated TLV in HELLO payload")]
    TruncatedTlv,

    #[error("HELLO payload missing codec id")]
    MissingCodecId,

    #[error("payload too short for {0} message")]
    ShortPayload(&'static str),
}

/// A parsed message header. `msg_type` and `stream` are kept raw so unknown
/// values can be skipped (forward compatibility) instead of erroring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub msg_type: u8,
    pub stream: u8,
    pub flags: u8,
    pub sequence: u32,
    pub generation: u32,
    pub pts_us: i64,
    pub payload_len: usize,
}

fn read_u32(buf: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(buf[at..at + 4].try_into().expect("4 bytes"))
}

/// Parse a fixed message header. Fails when the version is unsupported or the
/// length field is impossible (shorter than the fixed header remainder or
/// above the cap) — both mean the byte stream cannot be trusted further.
pub fn parse_header(buf: &[u8; HEADER_SIZE]) -> Result<Header, ProtocolError> {
    let length = read_u32(buf, 0);
    let version = buf[4];
    if version != PROTOCOL_VERSION {
        return Err(ProtocolError::UnsupportedVersion(version));
    }
    if !(HEADER_LENGTH_BYTES..=MAX_MESSAGE_LENGTH).contains(&length) {
        return Err(ProtocolError::BadLength(length));
    }
    Ok(Header {
        msg_type: buf[5],
        stream: buf[6],
        flags: buf[7],
        sequence: read_u32(buf, 8),
        generation: read_u32(buf, 12),
        pts_us: u64::from_le_bytes(buf[16..24].try_into().expect("8 bytes")) as i64,
        payload_len: (length - HEADER_LENGTH_BYTES) as usize,
    })
}

/// Decoded HELLO parameters. Zero / empty means "not present on the wire"
/// for every field except `codec_id`, which is required.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HelloInfo {
    /// ffmpeg `AVCodecID` value
    pub codec_id: u32,
    /// Raw `codecpar->extradata`: SPS/PPS/VPS for H.26x (Annex B or
    /// avcC/hvcC), AudioSpecificConfig for AAC.
    pub extradata: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub fps_num: u32,
    pub fps_den: u32,
    pub sample_rate: u32,
    pub channels: u32,
    pub profile: u32,
    pub level: u32,
}

/// Parse a HELLO payload: a TLV list of (u8 tag, u16 length, value). Unknown
/// tags are skipped per the protocol spec.
pub fn parse_hello(data: &[u8]) -> Result<HelloInfo, ProtocolError> {
    let mut info = HelloInfo::default();
    let mut saw_codec_id = false;
    let mut i = 0usize;
    while i < data.len() {
        if i + 3 > data.len() {
            return Err(ProtocolError::TruncatedTlv);
        }
        let tag = data[i];
        let len = u16::from_le_bytes([data[i + 1], data[i + 2]]) as usize;
        i += 3;
        if i + len > data.len() {
            return Err(ProtocolError::TruncatedTlv);
        }
        let value = &data[i..i + len];
        i += len;

        let u32_value = || {
            if len == 4 {
                Some(read_u32(value, 0))
            } else {
                None
            }
        };
        match tag {
            TLV_CODEC_ID => {
                if let Some(v) = u32_value() {
                    info.codec_id = v;
                    saw_codec_id = true;
                }
            }
            TLV_EXTRADATA => info.extradata = value.to_vec(),
            TLV_WIDTH => info.width = u32_value().unwrap_or(0),
            TLV_HEIGHT => info.height = u32_value().unwrap_or(0),
            TLV_FPS_NUM => info.fps_num = u32_value().unwrap_or(0),
            TLV_FPS_DEN => info.fps_den = u32_value().unwrap_or(0),
            TLV_SAMPLE_RATE => info.sample_rate = u32_value().unwrap_or(0),
            TLV_CHANNELS => info.channels = u32_value().unwrap_or(0),
            TLV_PROFILE => info.profile = u32_value().unwrap_or(0),
            TLV_LEVEL => info.level = u32_value().unwrap_or(0),
            _ => {} // unknown tag: skip
        }
    }
    if !saw_codec_id {
        return Err(ProtocolError::MissingCodecId);
    }
    Ok(info)
}

/// Parse a STATS payload: u64 messages sent, u64 messages dropped for this
/// consumer.
pub fn parse_stats(data: &[u8]) -> Result<(u64, u64), ProtocolError> {
    if data.len() < 16 {
        return Err(ProtocolError::ShortPayload("STATS"));
    }
    let sent = u64::from_le_bytes(data[0..8].try_into().expect("8 bytes"));
    let dropped = u64::from_le_bytes(data[8..16].try_into().expect("8 bytes"));
    Ok((sent, dropped))
}

/// Map an ffmpeg `AVCodecID` from a video HELLO to the pipeline's codec enum.
pub fn video_codec_from_id(codec_id: u32) -> VideoCodec {
    use ffmpeg_next::ffi::AVCodecID;
    match codec_id {
        x if x == AVCodecID::AV_CODEC_ID_H264 as u32 => VideoCodec::H264,
        x if x == AVCodecID::AV_CODEC_ID_HEVC as u32 => VideoCodec::H265,
        _ => VideoCodec::Unknown,
    }
}

/// Map an ffmpeg `AVCodecID` from an audio HELLO to the pipeline's codec enum.
pub fn audio_codec_from_id(codec_id: u32) -> AudioCodec {
    use ffmpeg_next::ffi::AVCodecID;
    match codec_id {
        x if x == AVCodecID::AV_CODEC_ID_AAC as u32 => AudioCodec::Aac,
        x if x == AVCodecID::AV_CODEC_ID_PCM_ALAW as u32 => AudioCodec::G711Alaw,
        x if x == AVCodecID::AV_CODEC_ID_PCM_MULAW as u32 => AudioCodec::G711Ulaw,
        x if x == AVCodecID::AV_CODEC_ID_OPUS as u32 => AudioCodec::Opus,
        _ => AudioCodec::Unknown,
    }
}

/// Test helpers that build wire messages the way zmc's encoder does. Used by
/// the reader and router tests to run a scripted fake zmc.
#[cfg(test)]
pub(crate) mod test_encode {
    use super::*;

    /// Serialize one complete message (header + payload).
    pub fn encode_message(
        msg_type: u8,
        stream: u8,
        flags: u8,
        sequence: u32,
        generation: u32,
        pts_us: i64,
        payload: &[u8],
    ) -> Vec<u8> {
        let mut out = Vec::with_capacity(HEADER_SIZE + payload.len());
        out.extend_from_slice(&(HEADER_LENGTH_BYTES + payload.len() as u32).to_le_bytes());
        out.push(PROTOCOL_VERSION);
        out.push(msg_type);
        out.push(stream);
        out.push(flags);
        out.extend_from_slice(&sequence.to_le_bytes());
        out.extend_from_slice(&generation.to_le_bytes());
        out.extend_from_slice(&(pts_us as u64).to_le_bytes());
        out.extend_from_slice(payload);
        out
    }

    pub fn tlv(tag: u8, value: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(3 + value.len());
        out.push(tag);
        out.extend_from_slice(&(value.len() as u16).to_le_bytes());
        out.extend_from_slice(value);
        out
    }

    pub fn tlv_u32(tag: u8, value: u32) -> Vec<u8> {
        tlv(tag, &value.to_le_bytes())
    }

    /// Build a HELLO payload with a codec id and optional extradata.
    pub fn hello_payload(codec_id: u32, extradata: &[u8]) -> Vec<u8> {
        let mut p = tlv_u32(TLV_CODEC_ID, codec_id);
        if !extradata.is_empty() {
            p.extend_from_slice(&tlv(TLV_EXTRADATA, extradata));
        }
        p
    }

    pub fn h264_codec_id() -> u32 {
        ffmpeg_next::ffi::AVCodecID::AV_CODEC_ID_H264 as u32
    }

    pub fn h265_codec_id() -> u32 {
        ffmpeg_next::ffi::AVCodecID::AV_CODEC_ID_HEVC as u32
    }

    pub fn aac_codec_id() -> u32 {
        ffmpeg_next::ffi::AVCodecID::AV_CODEC_ID_AAC as u32
    }

    pub fn alaw_codec_id() -> u32 {
        ffmpeg_next::ffi::AVCodecID::AV_CODEC_ID_PCM_ALAW as u32
    }
}

#[cfg(test)]
mod tests {
    use super::test_encode::*;
    use super::*;

    #[test]
    fn header_roundtrip() {
        let msg = encode_message(0x02, 0, FLAG_KEYFRAME, 7, 2, 1_234_567, &[0xAA, 0xBB]);
        assert_eq!(msg.len(), HEADER_SIZE + 2);
        let header = parse_header(msg[..HEADER_SIZE].try_into().unwrap()).unwrap();
        assert_eq!(header.msg_type, 0x02);
        assert_eq!(header.stream, 0);
        assert_eq!(header.flags, FLAG_KEYFRAME);
        assert_eq!(header.sequence, 7);
        assert_eq!(header.generation, 2);
        assert_eq!(header.pts_us, 1_234_567);
        assert_eq!(header.payload_len, 2);
        assert_eq!(
            MessageType::from_u8(header.msg_type),
            Some(MessageType::Media)
        );
        assert_eq!(StreamId::from_u8(header.stream), Some(StreamId::Video));
    }

    #[test]
    fn header_rejects_bad_version() {
        let mut msg = encode_message(0x02, 0, 0, 0, 0, 0, &[]);
        msg[4] = 9;
        assert_eq!(
            parse_header(msg[..HEADER_SIZE].try_into().unwrap()),
            Err(ProtocolError::UnsupportedVersion(9))
        );
    }

    #[test]
    fn header_rejects_implausible_length() {
        // Length below the fixed-header remainder.
        let mut msg = encode_message(0x02, 0, 0, 0, 0, 0, &[]);
        msg[..4].copy_from_slice(&10u32.to_le_bytes());
        assert_eq!(
            parse_header(msg[..HEADER_SIZE].try_into().unwrap()),
            Err(ProtocolError::BadLength(10))
        );
        // Length above the cap.
        let huge = MAX_MESSAGE_LENGTH + 1;
        msg[..4].copy_from_slice(&huge.to_le_bytes());
        assert_eq!(
            parse_header(msg[..HEADER_SIZE].try_into().unwrap()),
            Err(ProtocolError::BadLength(huge))
        );
    }

    #[test]
    fn message_and_stream_ids() {
        assert_eq!(MessageType::from_u8(0x01), Some(MessageType::Hello));
        assert_eq!(MessageType::from_u8(0x03), Some(MessageType::Keyframe));
        assert_eq!(MessageType::from_u8(0x04), Some(MessageType::Stats));
        assert_eq!(MessageType::from_u8(0x05), Some(MessageType::Bye));
        assert_eq!(MessageType::from_u8(0x77), None);
        assert_eq!(StreamId::from_u8(1), Some(StreamId::Audio));
        assert_eq!(StreamId::from_u8(2), None);
    }

    #[test]
    fn hello_parses_codec_and_extradata() {
        let payload = hello_payload(h264_codec_id(), &[0x00, 0x00, 0x00, 0x01, 0x67]);
        let info = parse_hello(&payload).unwrap();
        assert_eq!(info.codec_id, h264_codec_id());
        assert_eq!(info.extradata, vec![0x00, 0x00, 0x00, 0x01, 0x67]);
        assert_eq!(video_codec_from_id(info.codec_id), VideoCodec::H264);
    }

    #[test]
    fn hello_parses_all_u32_fields() {
        let mut payload = tlv_u32(0x01, aac_codec_id());
        payload.extend_from_slice(&tlv_u32(0x03, 1920));
        payload.extend_from_slice(&tlv_u32(0x04, 1080));
        payload.extend_from_slice(&tlv_u32(0x05, 30));
        payload.extend_from_slice(&tlv_u32(0x06, 1));
        payload.extend_from_slice(&tlv_u32(0x07, 16000));
        payload.extend_from_slice(&tlv_u32(0x08, 1));
        payload.extend_from_slice(&tlv_u32(0x09, 2));
        payload.extend_from_slice(&tlv_u32(0x0A, 120));
        let info = parse_hello(&payload).unwrap();
        assert_eq!(info.width, 1920);
        assert_eq!(info.height, 1080);
        assert_eq!(info.fps_num, 30);
        assert_eq!(info.fps_den, 1);
        assert_eq!(info.sample_rate, 16000);
        assert_eq!(info.channels, 1);
        assert_eq!(info.profile, 2);
        assert_eq!(info.level, 120);
    }

    #[test]
    fn hello_skips_unknown_tags() {
        let mut payload = tlv(0x7F, &[1, 2, 3, 4, 5]); // unknown tag first
        payload.extend_from_slice(&tlv_u32(0x01, h265_codec_id()));
        let info = parse_hello(&payload).unwrap();
        assert_eq!(video_codec_from_id(info.codec_id), VideoCodec::H265);
    }

    #[test]
    fn hello_rejects_truncation_and_missing_codec() {
        // Tag + declared length running past the end.
        let mut payload = tlv_u32(0x01, h264_codec_id());
        payload.extend_from_slice(&[0x02, 0xFF, 0x00, 0xAA]); // claims 255 bytes, has 1
        assert_eq!(parse_hello(&payload), Err(ProtocolError::TruncatedTlv));
        // Tag header itself truncated.
        assert_eq!(parse_hello(&[0x01, 0x04]), Err(ProtocolError::TruncatedTlv));
        // No codec id at all.
        assert_eq!(
            parse_hello(&tlv_u32(0x03, 640)),
            Err(ProtocolError::MissingCodecId)
        );
        // Empty payload.
        assert_eq!(parse_hello(&[]), Err(ProtocolError::MissingCodecId));
    }

    #[test]
    fn stats_parse() {
        let mut payload = 1000u64.to_le_bytes().to_vec();
        payload.extend_from_slice(&42u64.to_le_bytes());
        assert_eq!(parse_stats(&payload), Ok((1000, 42)));
        assert_eq!(
            parse_stats(&[0u8; 8]),
            Err(ProtocolError::ShortPayload("STATS"))
        );
    }

    #[test]
    fn codec_id_mapping() {
        use ffmpeg_next::ffi::AVCodecID;
        assert_eq!(
            video_codec_from_id(AVCodecID::AV_CODEC_ID_H264 as u32),
            VideoCodec::H264
        );
        assert_eq!(
            video_codec_from_id(AVCodecID::AV_CODEC_ID_HEVC as u32),
            VideoCodec::H265
        );
        // AV1 is real but unsupported by this pipeline.
        assert_eq!(
            video_codec_from_id(AVCodecID::AV_CODEC_ID_AV1 as u32),
            VideoCodec::Unknown
        );
        assert_eq!(
            audio_codec_from_id(AVCodecID::AV_CODEC_ID_AAC as u32),
            AudioCodec::Aac
        );
        assert_eq!(
            audio_codec_from_id(AVCodecID::AV_CODEC_ID_PCM_ALAW as u32),
            AudioCodec::G711Alaw
        );
        assert_eq!(
            audio_codec_from_id(AVCodecID::AV_CODEC_ID_PCM_MULAW as u32),
            AudioCodec::G711Ulaw
        );
        assert_eq!(
            audio_codec_from_id(AVCodecID::AV_CODEC_ID_OPUS as u32),
            AudioCodec::Opus
        );
        assert_eq!(
            audio_codec_from_id(AVCodecID::AV_CODEC_ID_MP3 as u32),
            AudioCodec::Unknown
        );
    }
}
