//! Live SRTP video pump for one HomeKit stream session.
//!
//! Reuses the existing pipeline: subscribes to the monitor's H.264 broadcast
//! ([`SourceRouter::subscribe_video`]), seeds the cached SPS/PPS+IDR so the
//! controller can decode immediately, RTP-packetizes each access unit
//! (RFC 6184 via [`rtp::codecs::h264::H264Payloader`]), encrypts with SRTP
//! ([`srtp::context::Context::encrypt_rtp`]), and sends UDP to the controller's
//! negotiated address. No transcode — the H.264 is passed through.
//!
//! Phase 1 is video-only; audio negotiation succeeds but no audio RTP is sent.

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use bytes::Bytes;
use rtp::codecs::h264::H264Payloader;
use rtp::header::Header;
use rtp::packetizer::Payloader;
use tokio::net::UdpSocket;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};
use webrtc_srtp::context::Context;
use webrtc_srtp::protection_profile::ProtectionProfile;
use webrtc_util::Marshal;

use crate::streaming::source::SourceRouter;

use super::SrtpParams;

/// RTP payload type for H.264 (dynamic; HomeKit accepts the value we advertise).
const H264_PAYLOAD_TYPE: u8 = 99;
/// 90 kHz RTP clock for video.
const VIDEO_CLOCK_HZ: u64 = 90_000;
/// Conservative RTP MTU for SRTP-over-UDP payloads.
const RTP_MTU: usize = 1200;

/// Parameters captured at SetupEndpoints time, needed to start the pump.
pub struct StreamParams {
    pub monitor_id: u32,
    pub controller_ip: IpAddr,
    pub video_port: u16,
    pub video_ssrc: u32,
    /// SRTP keys the **accessory** generated and returned in the SetupEndpoints
    /// response. SRTP keys each direction with the sender's key, so the
    /// accessory→controller video is encrypted with these and the controller
    /// decrypts with the same pair it received.
    pub video_srtp: SrtpParams,
}

/// A running (or runnable) HomeKit stream session for the camera.
pub struct StreamSession {
    params: StreamParams,
    /// UDP socket bound at SetupEndpoints time (its local port was advertised to
    /// the controller). Held here so the port can't be reallocated before the
    /// stream starts.
    socket: std::sync::Mutex<Option<std::net::UdpSocket>>,
    source_router: Arc<SourceRouter>,
    task: Mutex<Option<JoinHandle<()>>>,
}

impl StreamSession {
    pub fn new(
        params: StreamParams,
        socket: std::net::UdpSocket,
        source_router: Arc<SourceRouter>,
    ) -> Self {
        Self {
            params,
            socket: std::sync::Mutex::new(Some(socket)),
            source_router,
            task: Mutex::new(None),
        }
    }

    /// Start the video pump if not already running.
    pub async fn start(self: &Arc<Self>) {
        let mut guard = self.task.lock().await;
        if guard.as_ref().is_some_and(|h| !h.is_finished()) {
            return;
        }
        let this = Arc::clone(self);
        *guard = Some(tokio::spawn(async move {
            if let Err(e) = this.run().await {
                warn!(
                    monitor = this.params.monitor_id,
                    "homekit stream ended: {e}"
                );
            }
        }));
    }

    /// Stop the pump.
    pub async fn stop(&self) {
        if let Some(handle) = self.task.lock().await.take() {
            handle.abort();
        }
    }

    async fn run(&self) -> Result<(), String> {
        let p = &self.params;

        // SRTP context for the outbound (accessory → controller) video stream.
        if p.video_srtp.crypto_suite != 0 {
            return Err(format!(
                "unsupported SRTP crypto suite {}",
                p.video_srtp.crypto_suite
            ));
        }
        let mut srtp = Context::new(
            &p.video_srtp.master_key,
            &p.video_srtp.master_salt,
            ProtectionProfile::Aes128CmHmacSha1_80,
            None,
            None,
        )
        .map_err(|e| format!("srtp context: {e}"))?;

        // Reuse the socket bound (and port-advertised) at SetupEndpoints time.
        let std_socket = self
            .socket
            .lock()
            .unwrap()
            .take()
            .ok_or_else(|| "stream socket already taken".to_string())?;
        std_socket
            .set_nonblocking(true)
            .map_err(|e| format!("udp nonblocking: {e}"))?;
        let socket = UdpSocket::from_std(std_socket).map_err(|e| format!("udp adopt: {e}"))?;
        let dest = SocketAddr::new(p.controller_ip, p.video_port);

        // Subscribe before starting the reader so we capture parameter sets.
        let source = self
            .source_router
            .create_source(p.monitor_id)
            .await
            .map_err(|e| format!("create source: {e}"))?;
        let mut video_rx = source.subscribe_video();
        self.source_router
            .start_reader(p.monitor_id)
            .await
            .map_err(|e| format!("start reader: {e}"))?;

        info!(
            monitor = p.monitor_id,
            %dest, "homekit video stream started"
        );

        let mut payloader = H264Payloader::default();
        let mut seq: u16 = 0;
        let mut au: Vec<u8> = Vec::new();
        let mut au_pts_us: i64 = 0;
        let mut base_pts_us: Option<i64> = None;

        // Seed SPS/PPS + IDR so the controller decodes from frame one.
        if let Some(kf) = source.cached_keyframe() {
            base_pts_us = Some(kf.timestamp_us);
            seq = self
                .send_au(
                    &socket,
                    dest,
                    &mut srtp,
                    &mut payloader,
                    &kf.keyframe_au,
                    0,
                    seq,
                )
                .await?;
        }

        loop {
            match video_rx.recv().await {
                Ok(packet) => {
                    let base = *base_pts_us.get_or_insert(packet.timestamp_us);
                    // Flush the accumulated access unit when the timestamp advances.
                    if !au.is_empty() && packet.timestamp_us != au_pts_us {
                        let ts = rtp_timestamp(au_pts_us - base);
                        seq = self
                            .send_au(&socket, dest, &mut srtp, &mut payloader, &au, ts, seq)
                            .await?;
                        au.clear();
                    }
                    au_pts_us = packet.timestamp_us;
                    au.extend_from_slice(&packet.data);
                }
                Err(RecvError::Lagged(n)) => {
                    debug!(monitor = p.monitor_id, "homekit stream lagged {n} packets");
                }
                Err(RecvError::Closed) => {
                    return Err("source closed".to_string());
                }
            }
        }
    }

    /// Packetize one Annex-B access unit, SRTP-encrypt, and send each fragment.
    /// Returns the next sequence number.
    #[allow(clippy::too_many_arguments)]
    async fn send_au(
        &self,
        socket: &UdpSocket,
        dest: SocketAddr,
        srtp: &mut Context,
        payloader: &mut H264Payloader,
        au: &[u8],
        timestamp: u32,
        mut seq: u16,
    ) -> Result<u16, String> {
        let ssrc = self.params.video_ssrc;
        let fragments = payloader
            .payload(RTP_MTU, &Bytes::copy_from_slice(au))
            .map_err(|e| format!("h264 payload: {e}"))?;
        let last = fragments.len().saturating_sub(1);
        for (i, payload) in fragments.into_iter().enumerate() {
            let header = Header {
                version: 2,
                marker: i == last,
                payload_type: H264_PAYLOAD_TYPE,
                sequence_number: seq,
                timestamp,
                ssrc,
                ..Default::default()
            };
            let pkt = rtp::packet::Packet { header, payload };
            let marshaled = pkt.marshal().map_err(|e| format!("rtp marshal: {e}"))?;
            let encrypted = srtp
                .encrypt_rtp(&marshaled)
                .map_err(|e| format!("srtp encrypt: {e}"))?;
            socket
                .send_to(&encrypted, dest)
                .await
                .map_err(|e| format!("udp send: {e}"))?;
            seq = seq.wrapping_add(1);
        }
        Ok(seq)
    }
}

/// Convert a microsecond offset to a 90 kHz RTP timestamp (wrapping u32).
fn rtp_timestamp(offset_us: i64) -> u32 {
    let ticks = (offset_us.max(0) as u64).wrapping_mul(VIDEO_CLOCK_HZ) / 1_000_000;
    ticks as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rtp_timestamp_scales_to_90khz() {
        // 1 second → 90000 ticks.
        assert_eq!(rtp_timestamp(1_000_000), 90_000);
        // 33.367 ms (~1 frame @ 30fps) → ~3003 ticks.
        assert_eq!(rtp_timestamp(33_367), 3003);
        assert_eq!(rtp_timestamp(-5), 0);
    }
}
