//! RTP packets for H.264 (RFC 6184) and H.265 (RFC 7798) access units, and
//! their framing on an RTSP connection (interleaved binary data, RFC 2326
//! §10.12).
//!
//! A NAL unit that fits goes in one packet; a larger one is split into
//! fragmentation units (FU-A for H.264, FU for H.265). Every packet of an
//! access unit carries its timestamp, and the last one the marker bit.

use crate::streaming::source::media::VideoCodec;

/// Largest RTP payload. Interleaved TCP could carry more, but players that
/// reassemble into fixed buffers expect a datagram-sized packet.
pub const MAX_PAYLOAD: usize = 1400;

/// The dynamic payload type the SDP assigns to video.
pub const PAYLOAD_TYPE: u8 = 96;

/// RTP clock rate for video.
pub const CLOCK_RATE: u64 = 90_000;

/// Turns access units into RTP packets for one stream.
pub struct Packetizer {
    codec: VideoCodec,
    ssrc: u32,
    seq: u16,
}

impl Packetizer {
    pub fn new(codec: VideoCodec, ssrc: u32, first_seq: u16) -> Self {
        Self {
            codec,
            ssrc,
            seq: first_seq,
        }
    }

    /// The sequence number the next packet gets.
    pub fn next_seq(&self) -> u16 {
        self.seq
    }

    /// The RTP packets for one Annex B access unit.
    pub fn packetize(&mut self, access_unit: &[u8], timestamp: u32) -> Vec<Vec<u8>> {
        let mut payloads = Vec::new();
        for nal in nal_units(access_unit) {
            match self.codec {
                VideoCodec::H265 => fragment_h265(nal, &mut payloads),
                VideoCodec::H264 | VideoCodec::Unknown => fragment_h264(nal, &mut payloads),
            }
        }
        let last = payloads.len().saturating_sub(1);
        payloads
            .iter()
            .enumerate()
            .map(|(i, p)| self.packet(p, timestamp, i == last))
            .collect()
    }

    fn packet(&mut self, payload: &[u8], timestamp: u32, marker: bool) -> Vec<u8> {
        let mut out = Vec::with_capacity(12 + payload.len());
        out.push(0x80); // version 2, no padding, extension or CSRCs
        out.push(if marker { 0x80 } else { 0 } | PAYLOAD_TYPE);
        out.extend_from_slice(&self.seq.to_be_bytes());
        out.extend_from_slice(&timestamp.to_be_bytes());
        out.extend_from_slice(&self.ssrc.to_be_bytes());
        out.extend_from_slice(payload);
        self.seq = self.seq.wrapping_add(1);
        out
    }
}

/// The NAL units of an Annex B buffer, without start codes or trailing zero
/// bytes.
pub fn nal_units(data: &[u8]) -> Vec<&[u8]> {
    let mut starts = Vec::new();
    let mut i = 0;
    while i + 3 <= data.len() {
        if data[i] == 0 && data[i + 1] == 0 && data[i + 2] == 1 {
            starts.push(i + 3);
            i += 3;
        } else {
            i += 1;
        }
    }
    let mut out = Vec::with_capacity(starts.len());
    for (n, &start) in starts.iter().enumerate() {
        let end = starts.get(n + 1).map(|&s| s - 3).unwrap_or(data.len());
        let mut nal = &data[start..end];
        while let [rest @ .., 0] = nal {
            nal = rest;
        }
        if !nal.is_empty() {
            out.push(nal);
        }
    }
    out
}

fn fragment_h264(nal: &[u8], out: &mut Vec<Vec<u8>>) {
    if nal.len() <= MAX_PAYLOAD {
        out.push(nal.to_vec());
        return;
    }
    let indicator = (nal[0] & 0xE0) | 28;
    let nal_type = nal[0] & 0x1F;
    let chunks: Vec<&[u8]> = nal[1..].chunks(MAX_PAYLOAD - 2).collect();
    let last = chunks.len() - 1;
    for (i, chunk) in chunks.into_iter().enumerate() {
        let mut header = nal_type;
        if i == 0 {
            header |= 0x80;
        }
        if i == last {
            header |= 0x40;
        }
        let mut p = Vec::with_capacity(2 + chunk.len());
        p.push(indicator);
        p.push(header);
        p.extend_from_slice(chunk);
        out.push(p);
    }
}

fn fragment_h265(nal: &[u8], out: &mut Vec<Vec<u8>>) {
    if nal.len() <= MAX_PAYLOAD || nal.len() < 3 {
        out.push(nal.to_vec());
        return;
    }
    let nal_type = (nal[0] >> 1) & 0x3F;
    let payload_header = [(nal[0] & 0x81) | (49 << 1), nal[1]];
    let chunks: Vec<&[u8]> = nal[2..].chunks(MAX_PAYLOAD - 3).collect();
    let last = chunks.len() - 1;
    for (i, chunk) in chunks.into_iter().enumerate() {
        let mut header = nal_type;
        if i == 0 {
            header |= 0x80;
        }
        if i == last {
            header |= 0x40;
        }
        let mut p = Vec::with_capacity(3 + chunk.len());
        p.extend_from_slice(&payload_header);
        p.push(header);
        p.extend_from_slice(chunk);
        out.push(p);
    }
}

/// An RTP packet as interleaved data on `channel` of an RTSP connection.
pub fn interleave(channel: u8, packet: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + packet.len());
    out.push(b'$');
    out.push(channel);
    out.extend_from_slice(&(packet.len() as u16).to_be_bytes());
    out.extend_from_slice(packet);
    out
}

/// Microseconds since the first frame as a 90 kHz RTP timestamp from `base`.
pub fn rtp_timestamp(base: u32, first_us: i64, now_us: i64) -> u32 {
    let elapsed = (now_us - first_us).max(0) as u64;
    base.wrapping_add((elapsed * CLOCK_RATE / 1_000_000) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn annexb(nals: &[&[u8]]) -> Vec<u8> {
        let mut out = Vec::new();
        for nal in nals {
            out.extend_from_slice(&[0, 0, 0, 1]);
            out.extend_from_slice(nal);
        }
        out
    }

    #[test]
    fn splits_annexb_with_both_start_code_lengths() {
        let data = [
            0, 0, 0, 1, 0x67, 1, 2, 0, 0, 1, 0x68, 3, 0, 0, 0, 1, 0x65, 4, 0,
        ];
        assert_eq!(
            nal_units(&data),
            [&[0x67, 1, 2][..], &[0x68, 3][..], &[0x65, 4][..]]
        );
    }

    #[test]
    fn small_nals_go_one_per_packet_and_the_last_is_marked() {
        let mut p = Packetizer::new(VideoCodec::H264, 0xAABBCCDD, 65535);
        let pkts = p.packetize(&annexb(&[&[0x67, 1], &[0x68, 2], &[0x65, 3]]), 9000);
        assert_eq!(pkts.len(), 3);
        for (i, pkt) in pkts.iter().enumerate() {
            assert_eq!(pkt[0], 0x80);
            assert_eq!(pkt[1] & 0x7F, PAYLOAD_TYPE);
            assert_eq!(pkt[1] & 0x80 != 0, i == 2, "marker on the last only");
            assert_eq!(&pkt[4..8], &9000u32.to_be_bytes());
            assert_eq!(&pkt[8..12], &0xAABBCCDDu32.to_be_bytes());
        }
        // Sequence numbers wrap.
        assert_eq!(&pkts[0][2..4], &[0xFF, 0xFF]);
        assert_eq!(&pkts[1][2..4], &[0, 0]);
        assert_eq!(&pkts[2][12..], &[0x65, 3]);
        assert_eq!(p.next_seq(), 2);
    }

    #[test]
    fn a_large_h264_nal_becomes_fu_a() {
        let mut nal = vec![0x65];
        nal.extend((0..3000u32).map(|i| (i % 251) as u8 + 1));
        let mut p = Packetizer::new(VideoCodec::H264, 1, 0);
        let pkts = p.packetize(&annexb(&[&nal]), 0);
        assert_eq!(pkts.len(), 3);
        let mut rebuilt = vec![(pkts[0][12] & 0xE0) | (pkts[0][13] & 0x1F)];
        for (i, pkt) in pkts.iter().enumerate() {
            assert!(pkt.len() - 12 <= MAX_PAYLOAD);
            assert_eq!(pkt[12] & 0x1F, 28);
            assert_eq!(pkt[13] & 0x80 != 0, i == 0, "start bit");
            assert_eq!(pkt[13] & 0x40 != 0, i == 2, "end bit");
            rebuilt.extend_from_slice(&pkt[14..]);
        }
        assert_eq!(rebuilt, nal);
    }

    #[test]
    fn a_large_h265_nal_becomes_fu() {
        // IDR_W_RADL (19): header 0x26 0x01.
        let mut nal = vec![0x26, 0x01];
        nal.extend((0..2000u32).map(|i| (i % 200) as u8 + 1));
        let mut p = Packetizer::new(VideoCodec::H265, 1, 0);
        let pkts = p.packetize(&annexb(&[&nal]), 0);
        assert_eq!(pkts.len(), 2);
        let mut rebuilt = vec![0x26, 0x01];
        for (i, pkt) in pkts.iter().enumerate() {
            assert_eq!((pkt[12] >> 1) & 0x3F, 49);
            assert_eq!(pkt[13], 0x01);
            assert_eq!(pkt[14] & 0x3F, 19);
            assert_eq!(pkt[14] & 0x80 != 0, i == 0);
            assert_eq!(pkt[14] & 0x40 != 0, i == 1);
            rebuilt.extend_from_slice(&pkt[15..]);
        }
        assert_eq!(rebuilt, nal);
    }

    #[test]
    fn interleaved_frames_carry_channel_and_length() {
        assert_eq!(interleave(2, &[9, 8, 7]), [b'$', 2, 0, 3, 9, 8, 7]);
    }

    #[test]
    fn timestamps_run_at_90khz() {
        assert_eq!(rtp_timestamp(1000, 5_000_000, 6_000_000), 91_000);
        assert_eq!(rtp_timestamp(u32::MAX, 0, 1_000_000), 89_999);
        assert_eq!(rtp_timestamp(7, 10, 0), 7, "never goes backwards");
    }
}
