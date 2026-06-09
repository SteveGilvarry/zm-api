//! HomeKit camera RTP stream management (HAP spec ch. 10).
//!
//! Implements the four characteristics that negotiate a live SRTP stream:
//! - **SupportedVideoStreamConfiguration** / **SupportedAudioStreamConfiguration**
//!   / **SupportedRTPConfiguration** — static TLV blobs advertised in
//!   `/accessories` ([`supported_video`], [`supported_audio`], [`supported_rtp`]).
//! - **SetupEndpoints** — the controller posts its address + SRTP keys; we reply
//!   with ours ([`StreamManager::setup_endpoints`]).
//! - **SelectedRTPStreamConfiguration** — the controller starts/stops the stream
//!   ([`StreamManager::select_stream`]).
//!
//! The actual packet pump lives in [`stream`].

pub mod stream;

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::sync::Mutex;

use base64::Engine;

use super::tlv8::{TlvReader, TlvWriter};

/// SRTP crypto suite we support: `AES_CM_128_HMAC_SHA1_80` (value 0).
const SRTP_AES_CM_128_HMAC_SHA1_80: u8 = 0;

// --- SetupEndpoints / SelectedRTP TLV tags (HAP spec 10.x). ---
mod tag {
    // Top-level SupportedVideoStreamConfiguration
    pub const VIDEO_CODEC_CONFIG: u8 = 1;
    // Video codec config sub-tags
    pub const CODEC_TYPE: u8 = 1;
    pub const CODEC_PARAMS: u8 = 2;
    pub const VIDEO_ATTRS: u8 = 3;
    // Codec params sub-tags
    pub const PROFILE_ID: u8 = 1;
    pub const LEVEL: u8 = 2;
    pub const PACKETIZATION_MODE: u8 = 3;
    // Video attribute sub-tags
    pub const ATTR_WIDTH: u8 = 1;
    pub const ATTR_HEIGHT: u8 = 2;
    pub const ATTR_FRAMERATE: u8 = 3;

    // SupportedAudioStreamConfiguration
    pub const AUDIO_CODEC_CONFIG: u8 = 1;
    pub const COMFORT_NOISE: u8 = 2;
    pub const AUDIO_CHANNELS: u8 = 1;
    pub const AUDIO_BITRATE: u8 = 2;
    pub const AUDIO_SAMPLERATE: u8 = 3;

    // SupportedRTPConfiguration
    pub const SRTP_CRYPTO_SUITE: u8 = 2;

    // SetupEndpoints
    pub const SESSION_ID: u8 = 1;
    pub const STATUS: u8 = 2;
    pub const ADDRESS: u8 = 3;
    pub const SRTP_VIDEO: u8 = 4;
    pub const SRTP_AUDIO: u8 = 5;
    pub const SSRC_VIDEO: u8 = 6;
    pub const SSRC_AUDIO: u8 = 7;
    // Address sub-tags
    pub const IP_VERSION: u8 = 1;
    pub const IP_ADDRESS: u8 = 2;
    pub const VIDEO_RTP_PORT: u8 = 3;
    pub const AUDIO_RTP_PORT: u8 = 4;
    // SRTP param sub-tags
    pub const CRYPTO_SUITE: u8 = 1;
    pub const MASTER_KEY: u8 = 2;
    pub const MASTER_SALT: u8 = 3;

    // SelectedRTPStreamConfiguration
    pub const SESSION_CONTROL: u8 = 1;
    pub const CONTROL_SESSION_ID: u8 = 1;
    pub const CONTROL_COMMAND: u8 = 2;
}

/// H.264 profile advertised to HomeKit (we advertise all three; the source
/// stream's actual profile is passed through unchanged).
const H264_PROFILES: [u8; 3] = [0, 1, 2]; // baseline, main, high
const H264_LEVELS: [u8; 3] = [0, 1, 2]; // 3.1, 3.2, 4.0

/// Common resolutions advertised. HomeKit picks one in SelectedRTP; we pass the
/// source through regardless, so this is advisory.
const RESOLUTIONS: [(u16, u16, u8); 4] = [
    (1920, 1080, 30),
    (1280, 720, 30),
    (640, 480, 30),
    (320, 240, 30),
];

fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// Build the SupportedVideoStreamConfiguration TLV (base64) advertised in
/// `/accessories`.
pub fn supported_video() -> String {
    // codec params: profiles, levels, packetization mode 0
    let mut params = TlvWriter::new();
    for p in H264_PROFILES {
        params.push_u8(tag::PROFILE_ID, p);
    }
    for l in H264_LEVELS {
        params.push_u8(tag::LEVEL, l);
    }
    params.push_u8(tag::PACKETIZATION_MODE, 0);

    let mut config = TlvWriter::new();
    config.push_u8(tag::CODEC_TYPE, 0); // H.264
    config.push(tag::CODEC_PARAMS, params.as_bytes());
    for (w, h, fps) in RESOLUTIONS {
        let mut attr = TlvWriter::new();
        attr.push(tag::ATTR_WIDTH, &w.to_le_bytes());
        attr.push(tag::ATTR_HEIGHT, &h.to_le_bytes());
        attr.push_u8(tag::ATTR_FRAMERATE, fps);
        config.push(tag::VIDEO_ATTRS, attr.as_bytes());
    }

    let mut top = TlvWriter::new();
    top.push(tag::VIDEO_CODEC_CONFIG, config.as_bytes());
    b64(top.as_bytes())
}

/// Build the SupportedAudioStreamConfiguration TLV (base64).
///
/// Phase 1 advertises Opus but does not transmit audio; HomeKit still requires a
/// valid audio config to be present.
pub fn supported_audio() -> String {
    let mut params = TlvWriter::new();
    params.push_u8(tag::AUDIO_CHANNELS, 1);
    params.push_u8(tag::AUDIO_BITRATE, 0); // variable
    params.push_u8(tag::AUDIO_SAMPLERATE, 1); // 16 kHz

    let mut config = TlvWriter::new();
    config.push_u8(tag::CODEC_TYPE, 3); // Opus
    config.push(tag::CODEC_PARAMS, params.as_bytes());

    let mut top = TlvWriter::new();
    top.push(tag::AUDIO_CODEC_CONFIG, config.as_bytes());
    top.push_u8(tag::COMFORT_NOISE, 0);
    b64(top.as_bytes())
}

/// Build the SupportedRTPConfiguration TLV (base64).
pub fn supported_rtp() -> String {
    let mut top = TlvWriter::new();
    top.push_u8(tag::SRTP_CRYPTO_SUITE, SRTP_AES_CM_128_HMAC_SHA1_80);
    b64(top.as_bytes())
}

/// SRTP key material from one direction of a SetupEndpoints exchange.
#[derive(Debug, Clone)]
pub struct SrtpParams {
    pub crypto_suite: u8,
    pub master_key: Vec<u8>,
    pub master_salt: Vec<u8>,
}

/// A controller's posted SetupEndpoints request, parsed.
#[derive(Debug, Clone)]
pub struct EndpointRequest {
    pub session_id: Vec<u8>,
    pub controller_ip: IpAddr,
    pub video_port: u16,
    pub audio_port: u16,
    pub video_srtp: SrtpParams,
    pub audio_srtp: SrtpParams,
}

/// Parse a SetupEndpoints write payload. Returns `None` on malformed input.
pub fn parse_setup_endpoints(body: &[u8]) -> Option<EndpointRequest> {
    let r = TlvReader::parse(body)?;
    let session_id = r.get(tag::SESSION_ID)?.to_vec();

    let addr = TlvReader::parse(r.get(tag::ADDRESS)?)?;
    let ip_str = std::str::from_utf8(addr.get(tag::IP_ADDRESS)?).ok()?;
    let controller_ip: IpAddr = ip_str.parse().ok()?;
    let video_port = le_u16(addr.get(tag::VIDEO_RTP_PORT)?)?;
    let audio_port = le_u16(addr.get(tag::AUDIO_RTP_PORT)?)?;

    let video_srtp = parse_srtp(r.get(tag::SRTP_VIDEO)?)?;
    let audio_srtp = parse_srtp(r.get(tag::SRTP_AUDIO)?)?;

    Some(EndpointRequest {
        session_id,
        controller_ip,
        video_port,
        audio_port,
        video_srtp,
        audio_srtp,
    })
}

fn parse_srtp(body: &[u8]) -> Option<SrtpParams> {
    let r = TlvReader::parse(body)?;
    Some(SrtpParams {
        crypto_suite: r.get_u8(tag::CRYPTO_SUITE)?,
        master_key: r.get(tag::MASTER_KEY)?.to_vec(),
        master_salt: r.get(tag::MASTER_SALT)?.to_vec(),
    })
}

/// Build the SetupEndpoints read response that the controller fetches after its
/// write, echoing our accessory address, SRTP params, and SSRCs.
#[allow(clippy::too_many_arguments)]
pub fn build_setup_response(
    session_id: &[u8],
    accessory_ip: IpAddr,
    video_port: u16,
    audio_port: u16,
    video_srtp: &SrtpParams,
    audio_srtp: &SrtpParams,
    video_ssrc: u32,
    audio_ssrc: u32,
) -> Vec<u8> {
    let mut addr = TlvWriter::new();
    addr.push_u8(tag::IP_VERSION, if accessory_ip.is_ipv6() { 1 } else { 0 })
        .push(tag::IP_ADDRESS, accessory_ip.to_string().as_bytes())
        .push(tag::VIDEO_RTP_PORT, &video_port.to_le_bytes())
        .push(tag::AUDIO_RTP_PORT, &audio_port.to_le_bytes());

    let mut w = TlvWriter::new();
    w.push(tag::SESSION_ID, session_id)
        .push_u8(tag::STATUS, 0) // success
        .push(tag::ADDRESS, addr.as_bytes())
        .push(tag::SRTP_VIDEO, &srtp_tlv(video_srtp))
        .push(tag::SRTP_AUDIO, &srtp_tlv(audio_srtp))
        .push(tag::SSRC_VIDEO, &video_ssrc.to_le_bytes())
        .push(tag::SSRC_AUDIO, &audio_ssrc.to_le_bytes());
    w.into_bytes()
}

fn srtp_tlv(p: &SrtpParams) -> Vec<u8> {
    let mut w = TlvWriter::new();
    w.push_u8(tag::CRYPTO_SUITE, p.crypto_suite)
        .push(tag::MASTER_KEY, &p.master_key)
        .push(tag::MASTER_SALT, &p.master_salt);
    w.into_bytes()
}

/// The SelectedRTPStreamConfiguration command, parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamCommand {
    End,
    Start,
    Suspend,
    Resume,
    Reconfigure,
    Unknown(u8),
}

impl From<u8> for StreamCommand {
    fn from(v: u8) -> Self {
        match v {
            0 => StreamCommand::End,
            1 => StreamCommand::Start,
            2 => StreamCommand::Suspend,
            3 => StreamCommand::Resume,
            4 => StreamCommand::Reconfigure,
            other => StreamCommand::Unknown(other),
        }
    }
}

/// Parse a SelectedRTPStreamConfiguration write, returning `(session_id, command)`.
pub fn parse_selected_rtp(body: &[u8]) -> Option<(Vec<u8>, StreamCommand)> {
    let r = TlvReader::parse(body)?;
    let control = TlvReader::parse(r.get(tag::SESSION_CONTROL)?)?;
    let session_id = control.get(tag::CONTROL_SESSION_ID)?.to_vec();
    let command = StreamCommand::from(control.get_u8(tag::CONTROL_COMMAND)?);
    Some((session_id, command))
}

fn le_u16(b: &[u8]) -> Option<u16> {
    Some(u16::from_le_bytes([*b.first()?, *b.get(1)?]))
}

/// Tracks pending/active per-session stream negotiation for the one camera.
///
/// Keyed by the controller's session id (the UUID from SetupEndpoints).
#[derive(Default)]
pub struct StreamManager {
    sessions: Mutex<HashMap<Vec<u8>, Arc<stream::StreamSession>>>,
}

impl StreamManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&self, session_id: Vec<u8>, session: Arc<stream::StreamSession>) {
        self.sessions.lock().unwrap().insert(session_id, session);
    }

    pub fn get(&self, session_id: &[u8]) -> Option<Arc<stream::StreamSession>> {
        self.sessions.lock().unwrap().get(session_id).cloned()
    }

    pub fn remove(&self, session_id: &[u8]) -> Option<Arc<stream::StreamSession>> {
        self.sessions.lock().unwrap().remove(session_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_configs_are_valid_base64_tlv() {
        for s in [supported_video(), supported_audio(), supported_rtp()] {
            let raw = base64::engine::general_purpose::STANDARD
                .decode(&s)
                .unwrap();
            assert!(TlvReader::parse(&raw).is_some(), "valid TLV: {s}");
        }
    }

    #[test]
    fn setup_endpoints_round_trips() {
        // Build a controller request, parse it, and confirm fields survive.
        let mut addr = TlvWriter::new();
        addr.push_u8(tag::IP_VERSION, 0)
            .push(tag::IP_ADDRESS, b"192.168.1.50")
            .push(tag::VIDEO_RTP_PORT, &50000u16.to_le_bytes())
            .push(tag::AUDIO_RTP_PORT, &50002u16.to_le_bytes());
        let mut vsrtp = TlvWriter::new();
        vsrtp
            .push_u8(tag::CRYPTO_SUITE, 0)
            .push(tag::MASTER_KEY, &[1u8; 16])
            .push(tag::MASTER_SALT, &[2u8; 14]);
        let mut asrtp = TlvWriter::new();
        asrtp
            .push_u8(tag::CRYPTO_SUITE, 0)
            .push(tag::MASTER_KEY, &[3u8; 16])
            .push(tag::MASTER_SALT, &[4u8; 14]);
        let mut req = TlvWriter::new();
        req.push(tag::SESSION_ID, &[0xAB; 16])
            .push(tag::ADDRESS, addr.as_bytes())
            .push(tag::SRTP_VIDEO, vsrtp.as_bytes())
            .push(tag::SRTP_AUDIO, asrtp.as_bytes());

        let parsed = parse_setup_endpoints(req.as_bytes()).expect("parse");
        assert_eq!(parsed.session_id, vec![0xAB; 16]);
        assert_eq!(parsed.controller_ip.to_string(), "192.168.1.50");
        assert_eq!(parsed.video_port, 50000);
        assert_eq!(parsed.audio_port, 50002);
        assert_eq!(parsed.video_srtp.master_key, vec![1u8; 16]);
        assert_eq!(parsed.audio_srtp.master_salt, vec![4u8; 14]);

        // And the response is parseable with status success.
        let resp = build_setup_response(
            &parsed.session_id,
            "192.168.1.10".parse().unwrap(),
            50000,
            50002,
            &parsed.video_srtp,
            &parsed.audio_srtp,
            0x11223344,
            0x55667788,
        );
        let rr = TlvReader::parse(&resp).unwrap();
        assert_eq!(rr.get_u8(tag::STATUS), Some(0));
        assert_eq!(rr.get(tag::SESSION_ID).unwrap(), &[0xAB; 16]);
    }

    #[test]
    fn parses_start_and_end_commands() {
        for (cmd, expect) in [(1u8, StreamCommand::Start), (0u8, StreamCommand::End)] {
            let mut control = TlvWriter::new();
            control
                .push(tag::CONTROL_SESSION_ID, &[0xCD; 16])
                .push_u8(tag::CONTROL_COMMAND, cmd);
            let mut top = TlvWriter::new();
            top.push(tag::SESSION_CONTROL, control.as_bytes());
            let (sid, parsed) = parse_selected_rtp(top.as_bytes()).unwrap();
            assert_eq!(sid, vec![0xCD; 16]);
            assert_eq!(parsed, expect);
        }
    }
}
